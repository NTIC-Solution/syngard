//! Read-only inventory of a bundle root.
//!
//! The walk never follows symlinks, never executes or fetches anything, and
//! reports every entry it could not inspect as a [`CoverageIssue`] instead of
//! skipping it silently.

use std::fs::{self, File};
use std::io::Read as _;
use std::path::{Component, Path, PathBuf};

use serde::Serialize;

use crate::digest::Sha256Digest;
use crate::manifest::{Entry, EntryKind, LogicalPath};

/// Resource limits for one inventory. Exceeding any of them makes coverage
/// incomplete; it never truncates silently.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub max_file_bytes: u64,
    pub max_files: usize,
    pub max_depth: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_file_bytes: 8 * 1024 * 1024,
            max_files: 10_000,
            max_depth: 32,
        }
    }
}

/// Why an entry could not be fully inspected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "reason")]
pub enum IssueReason {
    Symlink,
    Hardlink { links: u64 },
    SpecialFile,
    NonUtf8Path,
    TooLarge { size: u64, limit: u64 },
    TooDeep { limit: usize },
    TooManyFiles { limit: usize },
    ChangedDuringRead,
    Io { kind: String },
}

/// An entry that prevents a complete inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CoverageIssue {
    /// Path relative to the root, lossily decoded for display only.
    pub path: String,
    #[serde(flatten)]
    pub reason: IssueReason,
}

/// Result of an inventory: the entries that were read and the issues that
/// make the inventory incomplete.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inventory {
    pub entries: Vec<Entry>,
    pub issues: Vec<CoverageIssue>,
}

impl Inventory {
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.issues.is_empty()
    }
}

/// Errors that prevent any inventory of the root.
#[derive(Debug, thiserror::Error)]
pub enum InventoryError {
    #[error("root {path} is not a directory")]
    NotADirectory { path: PathBuf },
    #[error("cannot read root {path}: {source}")]
    Root {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

/// Inventories every regular file under `root`.
pub fn scan(root: &Path, limits: Limits) -> Result<Inventory, InventoryError> {
    let meta = fs::symlink_metadata(root).map_err(|source| InventoryError::Root {
        path: root.to_path_buf(),
        source,
    })?;
    if !meta.is_dir() {
        return Err(InventoryError::NotADirectory {
            path: root.to_path_buf(),
        });
    }
    let mut walker = Walker {
        root,
        limits,
        inventory: Inventory {
            entries: Vec::new(),
            issues: Vec::new(),
        },
        files_seen: 0,
    };
    walker.visit_dir(root, 0);
    let mut inventory = walker.inventory;
    inventory.entries.sort_by(|a, b| a.path.cmp(&b.path));
    inventory.issues.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(inventory)
}

struct Walker<'a> {
    root: &'a Path,
    limits: Limits,
    inventory: Inventory,
    files_seen: usize,
}

impl Walker<'_> {
    fn issue(&mut self, path: &Path, reason: IssueReason) {
        let display = path
            .strip_prefix(self.root)
            .unwrap_or(path)
            .to_string_lossy()
            .into_owned();
        self.inventory.issues.push(CoverageIssue {
            path: if display.is_empty() {
                ".".to_owned()
            } else {
                display
            },
            reason,
        });
    }

    fn visit_dir(&mut self, dir: &Path, depth: usize) {
        if depth > self.limits.max_depth {
            self.issue(
                dir,
                IssueReason::TooDeep {
                    limit: self.limits.max_depth,
                },
            );
            return;
        }
        let read = match fs::read_dir(dir) {
            Ok(read) => read,
            Err(err) => return self.issue(dir, io_reason(&err)),
        };
        let mut children: Vec<PathBuf> = Vec::new();
        for item in read {
            match item {
                Ok(item) => children.push(item.path()),
                Err(err) => self.issue(dir, io_reason(&err)),
            }
        }
        children.sort();
        for child in children {
            self.visit(&child, depth);
        }
    }

    fn visit(&mut self, path: &Path, depth: usize) {
        let meta = match fs::symlink_metadata(path) {
            Ok(meta) => meta,
            Err(err) => return self.issue(path, io_reason(&err)),
        };
        let file_type = meta.file_type();
        if file_type.is_symlink() {
            return self.issue(path, IssueReason::Symlink);
        }
        if file_type.is_dir() {
            return self.visit_dir(path, depth + 1);
        }
        if !file_type.is_file() {
            return self.issue(path, IssueReason::SpecialFile);
        }
        self.files_seen += 1;
        if self.files_seen > self.limits.max_files {
            // Report the limit once; the remaining files stay uncovered.
            if self.files_seen == self.limits.max_files + 1 {
                self.issue(
                    path,
                    IssueReason::TooManyFiles {
                        limit: self.limits.max_files,
                    },
                );
            }
            return;
        }
        let links = hardlink_count(&meta);
        if links > 1 {
            return self.issue(path, IssueReason::Hardlink { links });
        }
        let Some(logical) = self.logical_path(path) else {
            return self.issue(path, IssueReason::NonUtf8Path);
        };
        if meta.len() > self.limits.max_file_bytes {
            return self.issue(
                path,
                IssueReason::TooLarge {
                    size: meta.len(),
                    limit: self.limits.max_file_bytes,
                },
            );
        }
        match self.read_bounded(path) {
            Ok(Some(bytes)) => {
                let kind = EntryKind::classify(&logical, is_executable(&meta));
                self.inventory.entries.push(Entry {
                    size: bytes.len() as u64,
                    sha256: Sha256Digest::of(&bytes),
                    path: logical,
                    kind,
                });
            }
            Ok(None) => self.issue(path, IssueReason::ChangedDuringRead),
            Err(err) => self.issue(path, io_reason(&err)),
        }
    }

    /// Reads at most `max_file_bytes`; returns `None` if the file grew past
    /// the limit after its metadata was read.
    fn read_bounded(&self, path: &Path) -> std::io::Result<Option<Vec<u8>>> {
        let file = File::open(path)?;
        let mut bytes = Vec::new();
        file.take(self.limits.max_file_bytes.saturating_add(1))
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 > self.limits.max_file_bytes {
            return Ok(None);
        }
        Ok(Some(bytes))
    }

    fn logical_path(&self, path: &Path) -> Option<LogicalPath> {
        let relative = path.strip_prefix(self.root).ok()?;
        let mut parts = Vec::new();
        for component in relative.components() {
            match component {
                Component::Normal(part) => parts.push(part.to_str()?),
                _ => return None,
            }
        }
        LogicalPath::new(parts.join("/"))
    }
}

fn io_reason(err: &std::io::Error) -> IssueReason {
    IssueReason::Io {
        kind: format!("{:?}", err.kind()),
    }
}

#[cfg(unix)]
fn hardlink_count(meta: &fs::Metadata) -> u64 {
    std::os::unix::fs::MetadataExt::nlink(meta)
}

#[cfg(not(unix))]
fn hardlink_count(_meta: &fs::Metadata) -> u64 {
    1
}

#[cfg(unix)]
fn is_executable(meta: &fs::Metadata) -> bool {
    std::os::unix::fs::PermissionsExt::mode(&meta.permissions()) & 0o111 != 0
}

#[cfg(not(unix))]
fn is_executable(_meta: &fs::Metadata) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &Path, rel: &str, bytes: &[u8]) {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create fixture dir");
        }
        fs::write(path, bytes).expect("write fixture");
    }

    #[test]
    fn inventories_sorted_entries() {
        let dir = tempfile::tempdir().expect("tempdir");
        write(dir.path(), "SKILL.md", b"# skill\n");
        write(dir.path(), "scripts/run.sh", b"echo hi\n");
        write(dir.path(), "a.md", b"ref\n");
        let inv = scan(dir.path(), Limits::default()).expect("scan");
        assert!(inv.is_complete());
        let paths: Vec<&str> = inv.entries.iter().map(|e| e.path.as_str()).collect();
        assert_eq!(paths, ["SKILL.md", "a.md", "scripts/run.sh"]);
    }

    #[test]
    fn oversized_file_is_incomplete_not_skipped() {
        let dir = tempfile::tempdir().expect("tempdir");
        write(dir.path(), "SKILL.md", &[b'x'; 16]);
        let limits = Limits {
            max_file_bytes: 8,
            ..Limits::default()
        };
        let inv = scan(dir.path(), limits).expect("scan");
        assert!(inv.entries.is_empty());
        assert_eq!(
            inv.issues.first().map(|i| &i.reason),
            Some(&IssueReason::TooLarge { size: 16, limit: 8 })
        );
    }

    #[test]
    fn depth_limit_is_reported() {
        let dir = tempfile::tempdir().expect("tempdir");
        write(dir.path(), "a/b/c/SKILL.md", b"x");
        let limits = Limits {
            max_depth: 1,
            ..Limits::default()
        };
        let inv = scan(dir.path(), limits).expect("scan");
        assert!(!inv.is_complete());
        assert!(matches!(
            inv.issues.first().map(|i| &i.reason),
            Some(IssueReason::TooDeep { limit: 1 })
        ));
    }

    #[test]
    fn file_count_limit_is_reported_once() {
        let dir = tempfile::tempdir().expect("tempdir");
        for name in ["a", "b", "c", "d"] {
            write(dir.path(), name, b"x");
        }
        let limits = Limits {
            max_files: 2,
            ..Limits::default()
        };
        let inv = scan(dir.path(), limits).expect("scan");
        assert_eq!(inv.entries.len(), 2);
        assert_eq!(inv.issues.len(), 1);
    }

    #[test]
    fn root_must_be_directory() {
        let dir = tempfile::tempdir().expect("tempdir");
        write(dir.path(), "file", b"x");
        assert!(matches!(
            scan(&dir.path().join("file"), Limits::default()),
            Err(InventoryError::NotADirectory { .. })
        ));
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_and_hardlinks_are_rejected() {
        let dir = tempfile::tempdir().expect("tempdir");
        write(dir.path(), "SKILL.md", b"x");
        std::os::unix::fs::symlink("/etc/hostname", dir.path().join("link")).expect("symlink");
        fs::hard_link(dir.path().join("SKILL.md"), dir.path().join("hard")).expect("hardlink");
        let inv = scan(dir.path(), Limits::default()).expect("scan");
        let reasons: Vec<&IssueReason> = inv.issues.iter().map(|i| &i.reason).collect();
        assert!(reasons.contains(&&IssueReason::Symlink));
        assert!(reasons.contains(&&IssueReason::Hardlink { links: 2 }));
        // Both hardlinked names are refused; neither is trusted.
        assert!(inv.entries.is_empty());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn non_utf8_names_are_incomplete() {
        use std::os::unix::ffi::OsStrExt as _;
        let dir = tempfile::tempdir().expect("tempdir");
        let name = std::ffi::OsStr::from_bytes(b"bad\xff.md");
        fs::write(dir.path().join(name), b"x").expect("write");
        let inv = scan(dir.path(), Limits::default()).expect("scan");
        assert_eq!(
            inv.issues.first().map(|i| &i.reason),
            Some(&IssueReason::NonUtf8Path)
        );
    }
}
