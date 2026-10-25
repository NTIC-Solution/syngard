//! Adversarial inventory cases through the binary: special files, symlinks,
//! hostile names and resource limits.
//!
//! Every case must end with a bounded exit and never read outside the bundle.

#![cfg(unix)]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "test helpers fail loudly on fixture setup errors and timeouts"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/benign-skill"
);
const DEADLINE: Duration = Duration::from_secs(20);

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

fn bundle() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("skill");
    copy_tree(Path::new(FIXTURE), &root);
    (dir, root)
}

/// Runs syngard and kills it if it exceeds `DEADLINE`, so a hang fails the
/// test instead of blocking CI.
fn syngard(args: &[&str]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_syngard"))
        .args(args)
        .env_clear()
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn syngard");
    let start = Instant::now();
    while child.try_wait().expect("poll child").is_none() {
        if start.elapsed() > DEADLINE {
            let _ = child.kill();
            panic!("syngard {args:?} did not finish within {DEADLINE:?}");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    child.wait_with_output().expect("collect output")
}

fn path(p: &Path) -> &str {
    p.to_str().expect("utf-8 temp path")
}

fn scan_json(root: &Path) -> (Option<i32>, serde_json::Value) {
    let out = syngard(&["scan", path(root), "--format", "json"]);
    let json = serde_json::from_slice(&out.stdout).expect("valid json report");
    (out.status.code(), json)
}

fn reasons(json: &serde_json::Value) -> Vec<String> {
    json["coverage_issues"]
        .as_array()
        .expect("issues array")
        .iter()
        .map(|i| i["reason"].as_str().expect("reason").to_owned())
        .collect()
}

fn mkfifo(path: &Path) {
    let status = Command::new("mkfifo")
        .arg(path)
        .status()
        .expect("run mkfifo");
    assert!(status.success(), "mkfifo failed");
}

/// a FIFO is never opened (opening it for reading would block).
#[test]
fn fifo_is_special_file_and_never_opened() {
    let (_dir, root) = bundle();
    mkfifo(&root.join("pipe.md"));
    let (code, json) = scan_json(&root);
    assert_eq!(code, Some(3));
    assert_eq!(json["complete"], false);
    assert_eq!(json["manifest"], serde_json::Value::Null);
    assert_eq!(reasons(&json), ["special_file"]);
}

/// a Unix socket in the bundle is refused like any special file.
#[test]
fn socket_is_special_file() {
    let (_dir, root) = bundle();
    let _listener =
        std::os::unix::net::UnixListener::bind(root.join("agent.sock")).expect("bind socket");
    let (code, json) = scan_json(&root);
    assert_eq!(code, Some(3));
    assert_eq!(reasons(&json), ["special_file"]);
}

/// a symlinked root is refused, never followed.
#[test]
fn symlinked_root_is_refused() {
    let (dir, root) = bundle();
    let link = dir.path().join("link-to-skill");
    std::os::unix::fs::symlink(&root, &link).expect("symlink root");
    let out = syngard(&["scan", path(&link), "--format", "json"]);
    assert_eq!(out.status.code(), Some(4));
    assert!(out.stdout.is_empty(), "no report for a refused root");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("is not a directory"), "{stderr}");
}

/// a symlinked directory inside the bundle is not traversed.
#[test]
fn symlinked_directory_is_not_traversed() {
    let (dir, root) = bundle();
    let outside = dir.path().join("outside");
    fs::create_dir(&outside).expect("outside dir");
    fs::write(outside.join("AGENTS.md"), "outside the bundle\n").expect("write");
    std::os::unix::fs::symlink(&outside, root.join("escape")).expect("symlink dir");
    let (code, json) = scan_json(&root);
    assert_eq!(code, Some(3));
    assert_eq!(reasons(&json), ["symlink"]);
    let text = serde_json::to_string(&json).expect("serialize");
    assert!(!text.contains("outside the bundle"));
    assert!(!text.contains("escape/AGENTS.md"));
}

/// terminal escapes and newlines in names are escaped in text output
/// and stay valid JSON.
#[test]
fn hostile_file_name_is_escaped() {
    let (_dir, root) = bundle();
    let name = "evil\u{1b}[2J\nfake.md";
    fs::write(root.join(name), "x\n").expect("write hostile name");

    let text = syngard(&["scan", path(&root)]);
    assert_eq!(text.status.code(), Some(0));
    assert!(!text.stdout.contains(&0x1b), "raw ESC byte in text output");
    let stdout = String::from_utf8_lossy(&text.stdout);
    assert!(stdout.contains(r"evil\u{1b}[2J\nfake.md"), "{stdout}");

    let (code, json) = scan_json(&root);
    assert_eq!(code, Some(0));
    let paths: Vec<&str> = json["manifest"]["entries"]
        .as_array()
        .expect("entries")
        .iter()
        .map(|e| e["path"].as_str().expect("path"))
        .collect();
    assert!(paths.contains(&name));
}

/// nesting beyond the depth limit makes the inspection incomplete.
#[test]
fn deep_nesting_is_incomplete() {
    let (_dir, root) = bundle();
    let mut deep = root.clone();
    for _ in 0..40 {
        deep.push("d");
    }
    fs::create_dir_all(&deep).expect("deep dirs");
    fs::write(deep.join("SKILL.md"), "deep\n").expect("write");
    let (code, json) = scan_json(&root);
    assert_eq!(code, Some(3));
    assert_eq!(reasons(&json), ["too_deep"]);
}

/// more files than the limit is reported once, never truncated silently.
#[test]
fn too_many_files_is_reported_once() {
    let (_dir, root) = bundle();
    let wide = root.join("wide");
    fs::create_dir(&wide).expect("wide dir");
    // The fixture has 2 files; the default limit is 10 000.
    for i in 0..10_000 {
        fs::write(wide.join(format!("{i:05}")), b"").expect("write");
    }
    let (code, json) = scan_json(&root);
    assert_eq!(code, Some(3));
    assert_eq!(reasons(&json), ["too_many_files"]);
}
