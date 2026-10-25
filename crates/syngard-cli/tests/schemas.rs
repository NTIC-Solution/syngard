//! Validate actual CLI reports against the versioned, offline JSON contracts.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

const SCHEMAS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../schemas");
const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/benign-skill"
);

struct ReportValidator {
    schemas: boon::Schemas,
    index: boon::SchemaIndex,
}

impl ReportValidator {
    fn is_valid(&self, report: &Value) -> bool {
        self.schemas.validate(report, self.index).is_ok()
    }
}

fn offline_compiler() -> boon::Compiler {
    let mut compiler = boon::Compiler::new();
    compiler.set_default_draft(boon::Draft::V2020_12);
    // An empty scheme loader rejects every unregistered HTTP or file resource.
    compiler.use_loader(Box::new(boon::SchemeUrlLoader::new()));
    compiler
}

fn validator(name: &str) -> TestResult<ReportValidator> {
    let mut compiler = offline_compiler();
    for file in [
        "manifest.v1.schema.json",
        "scan-report.v1.schema.json",
        "diff-report.v1.schema.json",
        "verify-report.v1.schema.json",
    ] {
        let schema: Value = serde_json::from_slice(&fs::read(Path::new(SCHEMAS).join(file))?)?;
        let id = schema
            .get("$id")
            .and_then(Value::as_str)
            .ok_or("missing schema ID")?
            .to_owned();
        compiler.add_resource(&id, schema)?;
    }
    let mut schemas = boon::Schemas::new();
    let index = compiler.compile(
        &format!("https://github.com/NTIC-Solution/syngard/schemas/{name}-report.v1.schema.json"),
        &mut schemas,
    )?;
    Ok(ReportValidator { schemas, index })
}

fn copy_tree(from: &Path, to: &Path) -> TestResult {
    fs::create_dir_all(to)?;
    for item in fs::read_dir(from)? {
        let item = item?;
        let target = to.join(item.file_name());
        if item.file_type()?.is_dir() {
            copy_tree(&item.path(), &target)?;
        } else {
            fs::copy(item.path(), target)?;
        }
    }
    Ok(())
}

struct Bundle {
    _dir: tempfile::TempDir,
    root: PathBuf,
    baseline: PathBuf,
}

impl Bundle {
    fn new() -> TestResult<Self> {
        let dir = tempfile::tempdir()?;
        let root = dir.path().join("skill");
        copy_tree(Path::new(FIXTURE), &root)?;
        let baseline = dir.path().join("baseline.json");
        let output = Command::new(env!("CARGO_BIN_EXE_syngard"))
            .arg("scan")
            .arg(&root)
            .arg("--output")
            .arg(&baseline)
            .env_clear()
            .output()?;
        assert_eq!(output.status.code(), Some(0));
        Ok(Self {
            _dir: dir,
            root,
            baseline,
        })
    }

    fn report(&self, name: &str, code: i32) -> TestResult<Value> {
        let mut command = Command::new(env!("CARGO_BIN_EXE_syngard"));
        command.arg(name).arg(&self.root).args(["--format", "json"]);
        match name {
            "diff" => {
                command.arg("--against").arg(&self.baseline);
            }
            "verify" => {
                command.arg("--baseline").arg(&self.baseline);
            }
            _ => {}
        }
        let output = command.env_clear().output()?;
        assert_eq!(
            output.status.code(),
            Some(code),
            "{name}: {:?}",
            output.stderr
        );
        let report: Value = serde_json::from_slice(&output.stdout)?;
        let validator = validator(name)?;
        if let Err(error) = validator.schemas.validate(&report, validator.index) {
            return Err(format!("{name}: {error:#}").into());
        }
        reject_object_mutations(&validator, &report, &report, "");
        Ok(report)
    }
}

/// Every serialized object is closed and every emitted field is required,
/// including nested manifest entries, change variants and coverage variants.
fn reject_object_mutations(
    validator: &ReportValidator,
    report: &Value,
    node: &Value,
    pointer: &str,
) {
    match node {
        Value::Object(fields) => {
            let mut extra = report.clone();
            if let Some(Value::Object(object)) = extra.pointer_mut(pointer) {
                object.insert("unexpected_field".into(), json!(true));
            }
            assert!(
                !validator.is_valid(&extra),
                "extra field accepted at {pointer}"
            );
            for (key, value) in fields {
                let mut missing = report.clone();
                if let Some(Value::Object(object)) = missing.pointer_mut(pointer) {
                    object.remove(key);
                }
                assert!(
                    !validator.is_valid(&missing),
                    "missing field accepted: {pointer}/{key}"
                );
                let escaped = key.replace('~', "~0").replace('/', "~1");
                reject_object_mutations(validator, report, value, &format!("{pointer}/{escaped}"));
            }
        }
        Value::Array(items) => {
            for (index, value) in items.iter().enumerate() {
                reject_object_mutations(validator, report, value, &format!("{pointer}/{index}"));
            }
        }
        _ => {}
    }
}

#[test]
fn complete_fixture_reports_match_schemas() -> TestResult {
    let bundle = Bundle::new()?;
    assert_eq!(bundle.report("scan", 0)?["complete"], true);
    assert_eq!(bundle.report("diff", 0)?["changes"], json!([]));
    assert_eq!(bundle.report("verify", 0)?["integrity"], "match");
    Ok(())
}

#[test]
fn drift_reports_cover_all_change_variants() -> TestResult {
    let bundle = Bundle::new()?;
    fs::write(bundle.root.join("SKILL.md"), "Changed inert instruction.\n")?;
    fs::remove_file(bundle.root.join("scripts/format.sh"))?;
    fs::write(bundle.root.join("added.txt"), "Inert reference.\n")?;
    bundle.report("scan", 0)?;
    for name in ["diff", "verify"] {
        let report = bundle.report(name, 2)?;
        let changes = report["changes"].as_array().ok_or("changes array")?;
        for kind in ["added", "removed", "modified"] {
            assert!(changes.iter().any(|change| change["change"] == kind));
        }
    }
    Ok(())
}

#[cfg(unix)]
#[test]
fn incomplete_symlink_reports_match_schemas() -> TestResult {
    let bundle = Bundle::new()?;
    std::os::unix::fs::symlink("SKILL.md", bundle.root.join("link.md"))?;
    let scan = bundle.report("scan", 3)?;
    assert_eq!(scan["complete"], false);
    assert!(scan["manifest"].is_null());
    assert_eq!(bundle.report("diff", 3)?["complete"], false);
    let verify = bundle.report("verify", 3)?;
    assert_eq!(verify["integrity"], "unknown");
    assert!(verify["observed_digest"].is_null());
    assert_eq!(verify["coverage_issues"][0]["reason"], "symlink");
    Ok(())
}

#[test]
fn empty_bundle_is_a_valid_boundary() -> TestResult {
    let bundle = Bundle::new()?;
    fs::remove_dir_all(&bundle.root)?;
    fs::create_dir(&bundle.root)?;
    assert_eq!(bundle.report("scan", 0)?["manifest"]["entries"], json!([]));
    bundle.report("diff", 2)?;
    bundle.report("verify", 2)?;
    Ok(())
}

#[test]
fn invalid_types_enums_and_digests_are_rejected() -> TestResult {
    let bundle = Bundle::new()?;
    for (name, pointer, value) in [
        ("scan", "/complete", json!("true")),
        ("scan", "/manifest/format", json!("syngard.manifest/v2")),
        ("scan", "/manifest/entries/0/size", json!(-1)),
        ("scan", "/manifest/entries/0/kind", json!("unexpected")),
        ("scan", "/manifest/entries/0/path", json!("../outside")),
        ("verify", "/integrity", json!("safe")),
        ("verify", "/approval", json!("approved")),
        ("verify", "/profile", json!("enforce")),
        ("verify", "/baseline_digest", json!("bad digest")),
        ("verify", "/observed_digest", json!("A".repeat(64))),
    ] {
        let mut report = bundle.report(name, 0)?;
        *report.pointer_mut(pointer).ok_or("mutation pointer")? = value;
        assert!(
            !validator(name)?.is_valid(&report),
            "accepted {name}{pointer}"
        );
    }
    Ok(())
}

#[test]
fn every_coverage_reason_matches_its_closed_variant() -> TestResult {
    use syngard_core::inventory::{CoverageIssue, IssueReason};

    let bundle = Bundle::new()?;
    for reason in [
        IssueReason::Symlink,
        IssueReason::Hardlink { links: 2 },
        IssueReason::SpecialFile,
        IssueReason::NonUtf8Path,
        IssueReason::TooLarge {
            size: u64::MAX,
            limit: 1,
        },
        IssueReason::TooDeep { limit: 32 },
        IssueReason::TooManyFiles { limit: 0 },
        IssueReason::ChangedDuringRead,
        IssueReason::Io {
            kind: "PermissionDenied".into(),
        },
    ] {
        let issue = serde_json::to_value(CoverageIssue {
            path: ".".into(),
            reason,
        })?;
        for name in ["scan", "diff", "verify"] {
            let mut report = bundle.report(name, 0)?;
            report["coverage_issues"] = json!([issue]);
            if name == "verify" {
                report["integrity"] = json!("unknown");
                report["observed_digest"] = Value::Null;
            } else {
                report["complete"] = json!(false);
                if name == "scan" {
                    report["manifest"] = Value::Null;
                }
            }
            let validator = validator(name)?;
            assert!(validator.is_valid(&report), "{name}: {issue}");
            reject_object_mutations(&validator, &report, &report, "");
            report["coverage_issues"][0]["reason"] = json!("unexpected");
            assert!(!validator.is_valid(&report));
        }
    }
    Ok(())
}

#[test]
fn contradictory_inspection_states_are_rejected() -> TestResult {
    let bundle = Bundle::new()?;
    for (name, pointer, value) in [
        ("scan", "/manifest", Value::Null),
        ("scan", "/complete", json!(false)),
        ("diff", "/complete", json!(false)),
        ("verify", "/observed_digest", Value::Null),
        ("verify", "/integrity", json!("unknown")),
    ] {
        let mut report = bundle.report(name, 0)?;
        *report.pointer_mut(pointer).ok_or("mutation pointer")? = value;
        assert!(
            !validator(name)?.is_valid(&report),
            "accepted {name}{pointer}"
        );
    }
    Ok(())
}

#[test]
fn manifest_path_rejects_traversal_after_a_newline() -> TestResult {
    let bundle = Bundle::new()?;
    let mut report = bundle.report("scan", 0)?;
    let invalid = "parent\n/../outside";
    assert!(syngard_core::manifest::LogicalPath::new(invalid.into()).is_none());
    report["manifest"]["entries"][0]["path"] = json!(invalid);
    assert!(!validator("scan")?.is_valid(&report));
    Ok(())
}

#[test]
fn manifest_paths_follow_logical_path_boundaries() -> TestResult {
    let bundle = Bundle::new()?;
    let report = bundle.report("scan", 0)?;
    let validator = validator("scan")?;
    for path in [
        "",
        "/abs",
        "a//b",
        "./a",
        "a/../b",
        "..",
        "a\\b",
        ".",
        "a/",
        "a/.",
        "a/..",
        "a\n/../b",
        "a\n//b",
        "\n/./b",
        "a/../\n",
        "file",
        "scripts/run.sh",
        ".hidden",
        "..hidden",
        "...",
        ".\n",
        "..\n",
        "a/..\n",
        "a\n/b",
        "\n",
        "é/文.md",
    ] {
        let mut mutated = report.clone();
        mutated["manifest"]["entries"][0]["path"] = json!(path);
        assert_eq!(
            validator.is_valid(&mutated),
            syngard_core::manifest::LogicalPath::new(path.into()).is_some(),
            "path: {path:?}"
        );
    }
    Ok(())
}

#[test]
fn unregistered_refs_fail_without_loading_http_or_files() -> TestResult {
    for target in [
        "https://example.invalid/missing.json",
        "file:///syngard-missing.json",
    ] {
        let mut compiler = offline_compiler();
        compiler.add_resource("https://example.invalid/root.json", json!({"$ref": target}))?;
        let mut schemas = boon::Schemas::new();
        assert!(
            compiler
                .compile("https://example.invalid/root.json", &mut schemas)
                .is_err()
        );
    }
    Ok(())
}
