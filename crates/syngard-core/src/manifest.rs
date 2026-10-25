//! Deterministic bundle manifest.
//!
//! The manifest digest is SHA-256 over the compact JSON of [`ManifestBody`]:
//! fields in declaration order, entries sorted by path bytes, digests in
//! lowercase hex. Adding, removing or changing any entry changes the digest.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::digest::Sha256Digest;

/// Format identifier written into every manifest.
pub const FORMAT: &str = "syngard.manifest/v1";

/// A `/`-separated path relative to the bundle root, without empty, `.` or
/// `..` segments.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct LogicalPath(String);

impl LogicalPath {
    pub fn new(path: String) -> Option<Self> {
        let valid = !path.is_empty()
            && path
                .split('/')
                .all(|seg| !seg.is_empty() && seg != "." && seg != ".." && !seg.contains('\\'));
        valid.then_some(Self(path))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn file_name(&self) -> &str {
        self.0.rsplit('/').next().unwrap_or(&self.0)
    }
}

impl fmt::Display for LogicalPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Debug for LogicalPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<'de> Deserialize<'de> for LogicalPath {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::new(text).ok_or_else(|| serde::de::Error::custom("invalid logical path"))
    }
}

/// Role of a file in a bundle. Classification is advisory for analysis; the
/// integrity decision covers every entry regardless of kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryKind {
    Instruction,
    Script,
    Reference,
    Config,
    Other,
}

impl EntryKind {
    /// Classifies by exact, case-sensitive file name, then extension.
    #[must_use]
    pub fn classify(path: &LogicalPath, executable: bool) -> Self {
        let name = path.file_name();
        if matches!(name, "AGENTS.md" | "CLAUDE.md" | "SKILL.md") {
            return Self::Instruction;
        }
        let ext = name.rsplit_once('.').map(|(_, ext)| ext);
        match ext {
            Some("sh" | "bash" | "zsh" | "py" | "js" | "mjs" | "ts" | "rb" | "pl" | "ps1") => {
                Self::Script
            }
            _ if executable => Self::Script,
            Some("md" | "txt") => Self::Reference,
            Some("json" | "toml" | "yaml" | "yml") => Self::Config,
            _ => Self::Other,
        }
    }
}

/// One file in a bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub path: LogicalPath,
    pub kind: EntryKind,
    pub size: u64,
    pub sha256: Sha256Digest,
}

/// The digested part of a manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestBody {
    pub format: String,
    pub entries: Vec<Entry>,
}

/// A manifest with its digest. Only built from a complete inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub digest: Sha256Digest,
    #[serde(flatten)]
    pub body: ManifestBody,
}

/// Errors when loading a manifest from untrusted bytes.
#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("manifest is not valid JSON for {FORMAT}: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("unsupported manifest format {0:?}")]
    Format(String),
    #[error("manifest entries are not strictly sorted by path")]
    Unsorted,
    #[error("manifest digest {stored} does not match its content ({computed})")]
    DigestMismatch {
        stored: Sha256Digest,
        computed: Sha256Digest,
    },
}

impl ManifestBody {
    /// Canonical bytes hashed into the manifest digest.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        // Serializing a struct of strings, integers and enums cannot fail.
        serde_json::to_vec(self).unwrap_or_default()
    }

    #[must_use]
    pub fn digest(&self) -> Sha256Digest {
        Sha256Digest::of(&self.canonical_bytes())
    }
}

impl Manifest {
    /// Builds a manifest; `entries` are sorted here so callers cannot produce
    /// two encodings of the same set.
    #[must_use]
    pub fn from_entries(mut entries: Vec<Entry>) -> Self {
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        let body = ManifestBody {
            format: FORMAT.to_owned(),
            entries,
        };
        Self {
            digest: body.digest(),
            body,
        }
    }

    /// Parses and re-verifies a stored manifest. A manifest whose digest does
    /// not match its entries is rejected, never repaired.
    pub fn from_json(bytes: &[u8]) -> Result<Self, ManifestError> {
        let manifest: Self = serde_json::from_slice(bytes)?;
        if manifest.body.format != FORMAT {
            return Err(ManifestError::Format(manifest.body.format));
        }
        let sorted = manifest
            .body
            .entries
            .windows(2)
            .all(|pair| matches!(pair, [a, b] if a.path < b.path));
        if !sorted {
            return Err(ManifestError::Unsorted);
        }
        let computed = manifest.body.digest();
        if computed != manifest.digest {
            return Err(ManifestError::DigestMismatch {
                stored: manifest.digest,
                computed,
            });
        }
        Ok(manifest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(path: &str, bytes: &[u8]) -> Entry {
        let path = LogicalPath::new(path.to_owned()).expect("valid path");
        Entry {
            kind: EntryKind::classify(&path, false),
            size: bytes.len() as u64,
            sha256: Sha256Digest::of(bytes),
            path,
        }
    }

    #[test]
    fn canonical_encoding_vector() {
        let manifest = Manifest::from_entries(vec![entry("SKILL.md", b"")]);
        let canonical = String::from_utf8(manifest.body.canonical_bytes()).expect("utf8");
        assert_eq!(
            canonical,
            concat!(
                r#"{"format":"syngard.manifest/v1","entries":[{"path":"SKILL.md","#,
                r#""kind":"instruction","size":0,"#,
                r#""sha256":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"}]}"#
            )
        );
        assert_eq!(
            manifest.digest,
            Sha256Digest::of(canonical.as_bytes()),
            "digest covers exactly the canonical bytes"
        );
    }

    #[test]
    fn order_of_input_does_not_matter() {
        let a = Manifest::from_entries(vec![entry("b", b"1"), entry("a", b"2")]);
        let b = Manifest::from_entries(vec![entry("a", b"2"), entry("b", b"1")]);
        assert_eq!(a.digest, b.digest);
    }

    #[test]
    fn composition_changes_digest() {
        let base = Manifest::from_entries(vec![entry("SKILL.md", b"x")]);
        let added = Manifest::from_entries(vec![entry("SKILL.md", b"x"), entry("run.sh", b"")]);
        let renamed = Manifest::from_entries(vec![entry("skill.md", b"x")]);
        assert_ne!(base.digest, added.digest);
        assert_ne!(base.digest, renamed.digest);
    }

    #[test]
    fn json_round_trip_and_tamper_detection() {
        let manifest = Manifest::from_entries(vec![entry("SKILL.md", b"x")]);
        let json = serde_json::to_vec(&manifest).expect("serialize");
        assert_eq!(Manifest::from_json(&json).expect("parse"), manifest);

        let tampered = String::from_utf8(json)
            .expect("utf8")
            .replace("\"size\":1", "\"size\":2");
        assert!(matches!(
            Manifest::from_json(tampered.as_bytes()),
            Err(ManifestError::DigestMismatch { .. })
        ));
    }

    #[test]
    fn rejects_unsorted_and_foreign_formats() {
        let mut manifest = Manifest::from_entries(vec![entry("a", b""), entry("b", b"")]);
        manifest.body.entries.reverse();
        manifest.digest = manifest.body.digest();
        let json = serde_json::to_vec(&manifest).expect("serialize");
        assert!(matches!(
            Manifest::from_json(&json),
            Err(ManifestError::Unsorted)
        ));

        let mut other = Manifest::from_entries(vec![]);
        other.body.format = "other/v9".to_owned();
        other.digest = other.body.digest();
        let json = serde_json::to_vec(&other).expect("serialize");
        assert!(matches!(
            Manifest::from_json(&json),
            Err(ManifestError::Format(_))
        ));
    }

    #[test]
    fn logical_path_rejects_traversal() {
        for bad in ["", "/abs", "a//b", "./a", "a/../b", "..", "a\\b"] {
            assert!(LogicalPath::new(bad.to_owned()).is_none(), "{bad:?}");
        }
        assert!(LogicalPath::new("scripts/run.sh".to_owned()).is_some());
    }

    #[test]
    fn classification_is_case_sensitive() {
        let kind =
            |p: &str| EntryKind::classify(&LogicalPath::new(p.to_owned()).expect("p"), false);
        assert_eq!(kind("sub/AGENTS.md"), EntryKind::Instruction);
        assert_eq!(kind("agents.md"), EntryKind::Reference);
        assert_eq!(kind("skills.md"), EntryKind::Reference);
        assert_eq!(kind("scripts/x.sh"), EntryKind::Script);
        assert_eq!(kind("config.toml"), EntryKind::Config);
        assert_eq!(kind("blob"), EntryKind::Other);
    }
}
