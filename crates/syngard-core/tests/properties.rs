//! Bounded property checks for the manifest's integrity contract.

use proptest::prelude::*;
use syngard_core::Sha256Digest;
use syngard_core::diff::diff;
use syngard_core::manifest::{Entry, EntryKind, LogicalPath, Manifest, ManifestError};

fn path() -> impl Strategy<Value = LogicalPath> {
    let segment = "[A-Za-z0-9._-]{1,12}".prop_filter("segments cannot be dot or dot-dot", |s| {
        s != "." && s != ".."
    });
    proptest::collection::vec(segment, 1..=3).prop_filter_map("valid logical path", |parts| {
        LogicalPath::new(parts.join("/"))
    })
}

fn kind() -> impl Strategy<Value = EntryKind> {
    prop_oneof![
        Just(EntryKind::Instruction),
        Just(EntryKind::Script),
        Just(EntryKind::Reference),
        Just(EntryKind::Config),
        Just(EntryKind::Other),
    ]
}

fn entries() -> impl Strategy<Value = Vec<(Entry, Vec<u8>)>> {
    proptest::collection::btree_map(
        path(),
        (kind(), proptest::collection::vec(any::<u8>(), 0..=32)),
        0..=8,
    )
    .prop_map(|files| {
        files
            .into_iter()
            .map(|(path, (kind, bytes))| {
                let entry = Entry {
                    path,
                    kind,
                    size: bytes.len() as u64,
                    sha256: Sha256Digest::of(&bytes),
                };
                (entry, bytes)
            })
            .collect()
    })
}

fn manifest(files: &[(Entry, Vec<u8>)]) -> Manifest {
    Manifest::from_entries(files.iter().map(|(entry, _)| entry.clone()).collect())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn input_permutations_preserve_digest(
        files in entries(), keys in proptest::collection::vec(any::<u64>(), 8),
    ) {
        let original = manifest(&files);
        let mut shuffled: Vec<_> = files.into_iter().zip(keys).collect();
        shuffled.sort_by_key(|(_, key)| *key);
        let shuffled: Vec<_> = shuffled.into_iter().map(|(file, _)| file).collect();
        prop_assert_eq!(&original, &manifest(&shuffled));
        let mut reversed = shuffled;
        reversed.reverse();
        prop_assert_eq!(original.digest, manifest(&reversed).digest);
    }

    #[test]
    fn every_entry_content_size_and_kind_are_covered(
        files in entries(), delta in 1u8..=u8::MAX, size in any::<u64>(),
    ) {
        let original = manifest(&files);
        for (index, (entry, bytes)) in files.iter().enumerate() {
            for position in 0..bytes.len() {
                let mut content = bytes.clone();
                if let Some(byte) = content.get_mut(position) {
                    *byte ^= delta;
                }
                let mut changed = original.body.entries.clone();
                if let Some(target) = changed.get_mut(index) {
                    target.sha256 = Sha256Digest::of(&content);
                }
                prop_assert_ne!(original.digest, Manifest::from_entries(changed).digest);
            }
            let mut changed = original.body.entries.clone();
            if let Some(target) = changed.get_mut(index) {
                target.size = if size == entry.size { size ^ 1 } else { size };
            }
            prop_assert_ne!(original.digest, Manifest::from_entries(changed).digest);
            for replacement in [EntryKind::Instruction, EntryKind::Script,
                EntryKind::Reference, EntryKind::Config, EntryKind::Other] {
                if replacement == entry.kind {
                    continue;
                }
                let mut changed = original.body.entries.clone();
                if let Some(target) = changed.get_mut(index) {
                    target.kind = replacement;
                }
                prop_assert_ne!(original.digest, Manifest::from_entries(changed).digest);
            }
        }
    }

    #[test]
    fn adding_and_removing_entries_changes_digest(files in entries(), extra in path()) {
        let original = manifest(&files);
        // A deeper path than the generator can produce is guaranteed unique.
        let Some(path) = LogicalPath::new(format!("added/{}/entry/file", extra.as_str())) else {
            return Err(TestCaseError::fail("generated addition path must be valid"));
        };
        let mut added = original.body.entries.clone();
        added.push(Entry { path, kind: EntryKind::Other, size: 0, sha256: Sha256Digest::of(b"") });
        prop_assert_ne!(original.digest, Manifest::from_entries(added).digest);
        for index in 0..files.len() {
            let mut removed = original.body.entries.clone();
            removed.remove(index);
            prop_assert_ne!(original.digest, Manifest::from_entries(removed).digest);
        }
    }

    #[test]
    fn empty_diff_exactly_matches_digest_equality(
        left in entries(), right in entries(), same in any::<bool>(),
    ) {
        let a = manifest(&left);
        let b = if same {
            let mut reversed = left;
            reversed.reverse();
            manifest(&reversed)
        } else {
            manifest(&right)
        };
        prop_assert_eq!(diff(&a, &b).is_empty(), a.digest == b.digest);
        prop_assert_eq!(diff(&b, &a).is_empty(), b.digest == a.digest);
    }

    #[test]
    fn json_round_trip_and_single_byte_mutations(
        files in entries(), delta in 1u8..=u8::MAX,
    ) {
        let original = manifest(&files);
        let json = serde_json::to_vec(&original)?;
        prop_assert_eq!(Manifest::from_json(&json).map_err(|e| TestCaseError::fail(e.to_string()))?, original.clone());
        // Visit every byte offset; the generated nonzero XOR explores replacements
        // across all 256 byte values, including invalid UTF-8 and JSON syntax.
        for position in 0..json.len() {
            let mut mutated = json.clone();
            if let Some(byte) = mutated.get_mut(position) {
                *byte ^= delta;
            }
            match Manifest::from_json(&mutated) {
                Ok(parsed) => prop_assert_eq!(&parsed, &original),
                Err(ManifestError::Parse(_) | ManifestError::DigestMismatch { .. }
                    | ManifestError::Unsorted | ManifestError::Format(_)) => {}
            }
        }
    }
}

#[test]
fn empty_manifest_and_maximum_size_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let empty = Manifest::from_entries(vec![]);
    assert_eq!(Manifest::from_json(&serde_json::to_vec(&empty)?)?, empty);
    let path = LogicalPath::new("a/._-09AZaz".to_owned()).ok_or("invalid test path")?;
    let maximum = Manifest::from_entries(vec![Entry {
        path,
        kind: EntryKind::Other,
        size: u64::MAX,
        sha256: Sha256Digest::of(b""),
    }]);
    assert_eq!(
        Manifest::from_json(&serde_json::to_vec(&maximum)?)?,
        maximum
    );
    Ok(())
}

#[test]
fn malformed_unsorted_and_foreign_manifests_are_rejected() -> Result<(), Box<dyn std::error::Error>>
{
    assert!(matches!(
        Manifest::from_json(b"{"),
        Err(ManifestError::Parse(_))
    ));
    let mut entries = Vec::new();
    for name in ["a", "b"] {
        entries.push(Entry {
            path: LogicalPath::new(name.to_owned()).ok_or("invalid test path")?,
            kind: EntryKind::Other,
            size: 0,
            sha256: Sha256Digest::of(b""),
        });
    }
    let mut manifest = Manifest::from_entries(entries);
    manifest.body.entries.reverse();
    manifest.digest = manifest.body.digest();
    assert!(matches!(
        Manifest::from_json(&serde_json::to_vec(&manifest)?),
        Err(ManifestError::Unsorted)
    ));
    manifest.body.format = "syngard.manifest/v0".to_owned();
    manifest.digest = manifest.body.digest();
    assert!(matches!(
        Manifest::from_json(&serde_json::to_vec(&manifest)?),
        Err(ManifestError::Format(_))
    ));
    let mut tampered = Manifest::from_entries(vec![]);
    tampered.digest = Sha256Digest::of(b"different");
    assert!(matches!(
        Manifest::from_json(&serde_json::to_vec(&tampered)?),
        Err(ManifestError::DigestMismatch { .. })
    ));
    Ok(())
}
