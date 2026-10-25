//! Entry-level difference between two manifests.

use serde::Serialize;

use crate::manifest::{Entry, Manifest};

/// One changed entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "change")]
pub enum Change {
    Added { entry: Entry },
    Removed { entry: Entry },
    Modified { before: Entry, after: Entry },
}

/// Changes from `before` to `after`, sorted by path.
#[must_use]
pub fn diff(before: &Manifest, after: &Manifest) -> Vec<Change> {
    let mut old = before.body.entries.iter().peekable();
    let mut new = after.body.entries.iter().peekable();
    let mut changes = Vec::new();
    loop {
        match (old.peek(), new.peek()) {
            (None, None) => break,
            (Some(a), None) => {
                changes.push(Change::Removed {
                    entry: (*a).clone(),
                });
                old.next();
            }
            (None, Some(b)) => {
                changes.push(Change::Added {
                    entry: (*b).clone(),
                });
                new.next();
            }
            (Some(a), Some(b)) => match a.path.cmp(&b.path) {
                std::cmp::Ordering::Less => {
                    changes.push(Change::Removed {
                        entry: (*a).clone(),
                    });
                    old.next();
                }
                std::cmp::Ordering::Greater => {
                    changes.push(Change::Added {
                        entry: (*b).clone(),
                    });
                    new.next();
                }
                std::cmp::Ordering::Equal => {
                    if a != b {
                        changes.push(Change::Modified {
                            before: (*a).clone(),
                            after: (*b).clone(),
                        });
                    }
                    old.next();
                    new.next();
                }
            },
        }
    }
    changes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::digest::Sha256Digest;
    use crate::manifest::{EntryKind, LogicalPath};

    fn entry(path: &str, bytes: &[u8]) -> Entry {
        Entry {
            path: LogicalPath::new(path.to_owned()).expect("path"),
            kind: EntryKind::Other,
            size: bytes.len() as u64,
            sha256: Sha256Digest::of(bytes),
        }
    }

    #[test]
    fn identical_manifests_have_no_changes() {
        let m = Manifest::from_entries(vec![entry("a", b"1")]);
        assert!(diff(&m, &m).is_empty());
    }

    #[test]
    fn reports_added_removed_modified() {
        let before = Manifest::from_entries(vec![entry("a", b"1"), entry("b", b"1")]);
        let after = Manifest::from_entries(vec![entry("b", b"2"), entry("c", b"1")]);
        let changes = diff(&before, &after);
        assert!(matches!(
            changes.as_slice(),
            [
                Change::Removed { .. },
                Change::Modified { .. },
                Change::Added { .. }
            ]
        ));
    }

    #[test]
    fn same_size_byte_change_is_modified() {
        let before = Manifest::from_entries(vec![entry("a", b"ab")]);
        let after = Manifest::from_entries(vec![entry("a", b"ba")]);
        assert_eq!(diff(&before, &after).len(), 1);
    }
}
