//! `syngard` command-line interface.
//!
//! Exit codes (scan, diff, verify): 0 complete and satisfied, 2 policy
//! violation (drift), 3 incomplete inspection, 4 usage or input error.

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use syngard_core::diff::{Change, diff};
use syngard_core::inventory::{self, CoverageIssue, Inventory, Limits};
use syngard_core::manifest::Manifest;
use syngard_core::verify::{Integrity, Verdict, verify};

const EXIT_OK: u8 = 0;
const EXIT_VIOLATION: u8 = 2;
const EXIT_INCOMPLETE: u8 = 3;
const EXIT_USAGE: u8 = 4;

#[derive(Debug, Parser)]
#[command(
    name = "syngard",
    version,
    about = "Integrity control for AI agent instructions."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Inventory a bundle and print its candidate manifest.
    Scan {
        root: PathBuf,
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
        /// Write the manifest as a baseline file. Refused when coverage is incomplete.
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Show changes between a baseline and the current bundle.
    Diff {
        root: PathBuf,
        #[arg(long)]
        against: PathBuf,
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
    /// Check that the current bundle matches a baseline. A match is not an approval.
    Verify {
        root: PathBuf,
        #[arg(long)]
        baseline: PathBuf,
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Format {
    Text,
    Json,
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            let code = if err.use_stderr() {
                EXIT_USAGE
            } else {
                EXIT_OK
            };
            let _ = err.print();
            return ExitCode::from(code);
        }
    };
    match run(cli.command) {
        Ok(code) => ExitCode::from(code),
        Err(message) => {
            eprintln!("syngard: {message}");
            ExitCode::from(EXIT_USAGE)
        }
    }
}

fn run(command: Command) -> Result<u8, String> {
    match command {
        Command::Scan {
            root,
            format,
            output,
        } => scan(&root, format, output.as_deref()),
        Command::Diff {
            root,
            against,
            format,
        } => run_diff(&root, &against, format),
        Command::Verify {
            root,
            baseline,
            format,
        } => run_verify(&root, &baseline, format),
    }
}

fn inventory_of(root: &Path) -> Result<Inventory, String> {
    inventory::scan(root, Limits::default()).map_err(|err| err.to_string())
}

fn load_baseline(path: &Path) -> Result<Manifest, String> {
    let bytes = fs::read(path).map_err(|err| format!("cannot read {}: {err}", path.display()))?;
    Manifest::from_json(&bytes).map_err(|err| format!("{}: {err}", path.display()))
}

#[derive(Serialize)]
struct ScanReport<'a> {
    complete: bool,
    manifest: Option<&'a Manifest>,
    coverage_issues: &'a [CoverageIssue],
}

fn scan(root: &Path, format: Format, output: Option<&Path>) -> Result<u8, String> {
    let inv = inventory_of(root)?;
    let complete = inv.is_complete();
    let manifest = complete.then(|| Manifest::from_entries(inv.entries.clone()));
    if let (Some(path), Some(manifest)) = (output, manifest.as_ref()) {
        write_new(path, manifest)?;
    }
    match format {
        Format::Json => print_json(&ScanReport {
            complete,
            manifest: manifest.as_ref(),
            coverage_issues: &inv.issues,
        })?,
        Format::Text => {
            for entry in &inv.entries {
                println!(
                    "{}  {:<11} {}",
                    entry.sha256,
                    kind_label(entry.kind),
                    entry.path.as_str().escape_debug()
                );
            }
            print_issues(&inv.issues);
            match &manifest {
                Some(m) => println!("manifest {}  ({} entries)", m.digest, m.body.entries.len()),
                None => println!("manifest: none (inspection incomplete)"),
            }
        }
    }
    if !complete {
        if output.is_some() {
            eprintln!("syngard: baseline not written: inspection incomplete");
        }
        return Ok(EXIT_INCOMPLETE);
    }
    Ok(EXIT_OK)
}

/// Writes a baseline without overwriting an existing file.
fn write_new(path: &Path, manifest: &Manifest) -> Result<(), String> {
    let mut json = serde_json::to_vec_pretty(manifest).map_err(|err| err.to_string())?;
    json.push(b'\n');
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|err| format!("cannot create {}: {err}", path.display()))?;
    file.write_all(&json)
        .and_then(|()| file.sync_all())
        .map_err(|err| format!("cannot write {}: {err}", path.display()))
}

#[derive(Serialize)]
struct DiffReport<'a> {
    complete: bool,
    changes: &'a [Change],
    coverage_issues: &'a [CoverageIssue],
}

fn run_diff(root: &Path, against: &Path, format: Format) -> Result<u8, String> {
    let baseline = load_baseline(against)?;
    let inv = inventory_of(root)?;
    let observed = Manifest::from_entries(inv.entries.clone());
    let changes = diff(&baseline, &observed);
    match format {
        Format::Json => print_json(&DiffReport {
            complete: inv.is_complete(),
            changes: &changes,
            coverage_issues: &inv.issues,
        })?,
        Format::Text => {
            print_changes(&changes);
            print_issues(&inv.issues);
        }
    }
    Ok(if !inv.is_complete() {
        EXIT_INCOMPLETE
    } else if changes.is_empty() {
        EXIT_OK
    } else {
        EXIT_VIOLATION
    })
}

fn run_verify(root: &Path, baseline: &Path, format: Format) -> Result<u8, String> {
    let baseline = load_baseline(baseline)?;
    let inv = inventory_of(root)?;
    let verdict = verify(&baseline, &inv);
    match format {
        Format::Json => print_json(&verdict)?,
        Format::Text => print_verdict(&verdict),
    }
    Ok(match verdict.integrity {
        Integrity::Match => EXIT_OK,
        Integrity::Drift => EXIT_VIOLATION,
        Integrity::Unknown => EXIT_INCOMPLETE,
    })
}

fn print_json<T: Serialize>(value: &T) -> Result<(), String> {
    let json = serde_json::to_string_pretty(value).map_err(|err| err.to_string())?;
    println!("{json}");
    Ok(())
}

fn print_verdict(verdict: &Verdict) {
    print_changes(&verdict.changes);
    print_issues(&verdict.coverage_issues);
    let integrity = match verdict.integrity {
        Integrity::Match => "match",
        Integrity::Drift => "drift",
        Integrity::Unknown => "unknown",
    };
    println!("integrity: {integrity}");
    println!("approval:  not evaluated (baseline is not an approval)");
    println!("profile:   observe");
}

// Paths come from hostile trees; text output escapes control characters.
fn print_changes(changes: &[Change]) {
    for change in changes {
        match change {
            Change::Added { entry } => println!("+ {}", entry.path.as_str().escape_debug()),
            Change::Removed { entry } => println!("- {}", entry.path.as_str().escape_debug()),
            Change::Modified { before, after } => {
                let path = after.path.as_str().escape_debug();
                println!("~ {path}  {} -> {}", before.sha256, after.sha256);
            }
        }
    }
}

fn print_issues(issues: &[CoverageIssue]) {
    for issue in issues {
        let path = issue.path.escape_debug();
        let reason = serde_json::to_string(&issue.reason).unwrap_or_default();
        println!("! {path}  {reason}");
    }
}

fn kind_label(kind: syngard_core::manifest::EntryKind) -> &'static str {
    use syngard_core::manifest::EntryKind;
    match kind {
        EntryKind::Instruction => "instruction",
        EntryKind::Script => "script",
        EntryKind::Reference => "reference",
        EntryKind::Config => "config",
        EntryKind::Other => "other",
    }
}
