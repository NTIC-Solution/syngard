//! End-to-end checks of the M1 slice: scan, diff and verify through the binary.

#![expect(
    clippy::expect_used,
    reason = "test helpers fail loudly on fixture setup errors"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/benign-skill"
);

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("create dir");
    for item in fs::read_dir(from).expect("read fixture") {
        let item = item.expect("entry");
        let target = to.join(item.file_name());
        if item.file_type().expect("type").is_dir() {
            copy_tree(&item.path(), &target);
        } else {
            fs::copy(item.path(), target).expect("copy");
        }
    }
}

struct Bundle {
    _dir: tempfile::TempDir,
    root: PathBuf,
    baseline: PathBuf,
}

/// Copies the fixture and writes its baseline with `syngard scan --output`.
fn bundle() -> Bundle {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("skill");
    copy_tree(Path::new(FIXTURE), &root);
    let baseline = dir.path().join("baseline.json");
    let out = syngard(&["scan", path(&root), "--output", path(&baseline)]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    Bundle {
        _dir: dir,
        root,
        baseline,
    }
}

fn syngard(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_syngard"))
        .args(args)
        .env_clear()
        .output()
        .expect("run syngard")
}

fn path(p: &Path) -> &str {
    p.to_str().expect("utf-8 temp path")
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn verify_json(b: &Bundle) -> (Option<i32>, serde_json::Value) {
    let out = syngard(&[
        "verify",
        path(&b.root),
        "--baseline",
        path(&b.baseline),
        "--format",
        "json",
    ]);
    let json = serde_json::from_slice(&out.stdout).expect("json verdict");
    (out.status.code(), json)
}

#[test]
fn unchanged_bundle_matches_but_is_not_approved() {
    let b = bundle();
    let (code, json) = verify_json(&b);
    assert_eq!(code, Some(0));
    assert_eq!(json["integrity"], "match");
    assert_eq!(json["approval"], "not_evaluated");
    assert_eq!(json["profile"], "observe");
}

#[test]
fn scan_is_deterministic() {
    let b = bundle();
    let first = syngard(&["scan", path(&b.root), "--format", "json"]);
    let second = syngard(&["scan", path(&b.root), "--format", "json"]);
    assert_eq!(first.stdout, second.stdout);
}

/// one byte changed, size and mtime preserved.
#[test]
fn same_size_same_mtime_change_is_drift() {
    let b = bundle();
    let file = b.root.join("SKILL.md");
    let mtime = fs::metadata(&file)
        .and_then(|m| m.modified())
        .expect("mtime");
    let mut bytes = fs::read(&file).expect("read");
    let last = bytes.iter().rposition(|&c| c == b'.').expect("a period");
    bytes[last] = b'!';
    fs::write(&file, &bytes).expect("write");
    fs::File::options()
        .write(true)
        .open(&file)
        .and_then(|f| f.set_modified(mtime))
        .expect("restore mtime");

    let (code, json) = verify_json(&b);
    assert_eq!(code, Some(2));
    assert_eq!(json["integrity"], "drift");
    assert_eq!(json["changes"][0]["change"], "modified");
    assert_eq!(json["changes"][0]["after"]["path"], "SKILL.md");
}

/// only the associated script changes; the bundle is invalidated.
#[test]
fn script_change_invalidates_bundle() {
    let b = bundle();
    fs::write(
        b.root.join("scripts/format.sh"),
        "#!/bin/sh\necho changed\n",
    )
    .expect("write");
    let (code, json) = verify_json(&b);
    assert_eq!(code, Some(2));
    assert_eq!(json["changes"][0]["after"]["path"], "scripts/format.sh");
    assert_eq!(json["changes"][0]["after"]["kind"], "script");
}

#[test]
fn added_and_removed_files_are_drift() {
    let b = bundle();
    fs::write(b.root.join("AGENTS.md"), "new instruction\n").expect("write");
    fs::remove_file(b.root.join("scripts/format.sh")).expect("remove");
    let out = syngard(&["diff", path(&b.root), "--against", path(&b.baseline)]);
    assert_eq!(out.status.code(), Some(2));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("+ AGENTS.md"), "{text}");
    assert!(text.contains("- scripts/format.sh"), "{text}");
}

/// a symlink makes the inspection incomplete, never a match.
#[cfg(unix)]
#[test]
fn symlink_makes_verify_unknown() {
    let b = bundle();
    std::os::unix::fs::symlink("/etc/passwd", b.root.join("extra.md")).expect("symlink");
    let (code, json) = verify_json(&b);
    assert_eq!(code, Some(3));
    assert_eq!(json["integrity"], "unknown");
    assert_eq!(json["coverage_issues"][0]["reason"], "symlink");
}

#[cfg(unix)]
#[test]
fn incomplete_scan_refuses_to_write_baseline() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("skill");
    copy_tree(Path::new(FIXTURE), &root);
    std::os::unix::fs::symlink("/etc/passwd", root.join("link")).expect("symlink");
    let baseline = dir.path().join("baseline.json");
    let out = syngard(&["scan", path(&root), "--output", path(&baseline)]);
    assert_eq!(out.status.code(), Some(3));
    assert!(!baseline.exists());
}

#[test]
fn tampered_baseline_is_rejected() {
    let b = bundle();
    let text = fs::read_to_string(&b.baseline).expect("read");
    let tampered = text.replacen("\"kind\": \"script\"", "\"kind\": \"other\"", 1);
    assert_ne!(text, tampered);
    fs::write(&b.baseline, tampered).expect("write");
    let out = syngard(&["verify", path(&b.root), "--baseline", path(&b.baseline)]);
    assert_eq!(out.status.code(), Some(4));
    assert!(stderr(&out).contains("does not match"), "{}", stderr(&out));
}

#[test]
fn scan_never_overwrites_existing_baseline() {
    let b = bundle();
    let before = fs::read(&b.baseline).expect("read");
    let out = syngard(&["scan", path(&b.root), "--output", path(&b.baseline)]);
    assert_eq!(out.status.code(), Some(4));
    assert_eq!(fs::read(&b.baseline).expect("read"), before);
}

#[test]
fn usage_errors_exit_4() {
    assert_eq!(syngard(&["verify"]).status.code(), Some(4));
    assert_eq!(syngard(&["bogus"]).status.code(), Some(4));
    assert_eq!(syngard(&["--version"]).status.code(), Some(0));
    let missing = syngard(&["scan", "/nonexistent/syngard-test"]);
    assert_eq!(missing.status.code(), Some(4));
}
