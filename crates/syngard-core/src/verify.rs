//! Verification decision against a baseline manifest.
//!
//! The decision keeps integrity and approval separate: a matching
//! baseline is not an approval. Approvals arrive with the control service (M2),
//! so `approval` is always [`Approval::NotEvaluated`] here.

use serde::Serialize;

use crate::diff::{Change, diff};
use crate::inventory::{CoverageIssue, Inventory};
use crate::manifest::Manifest;

/// Whether the inspected bytes match the baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Integrity {
    Match,
    Drift,
    /// Inspection was incomplete; neither match nor drift can be asserted
    /// for the whole bundle.
    Unknown,
}

/// Approval state. Only the control service may produce anything other than
/// `NotEvaluated`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Approval {
    NotEvaluated,
}

/// Enforcement profile in effect. M1 only observes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    Observe,
}

/// Full verification result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Verdict {
    pub integrity: Integrity,
    pub approval: Approval,
    pub profile: Profile,
    pub baseline_digest: crate::Sha256Digest,
    /// Digest of the observed bundle, present only when inspection was complete.
    pub observed_digest: Option<crate::Sha256Digest>,
    pub changes: Vec<Change>,
    pub coverage_issues: Vec<CoverageIssue>,
}

/// Compares an inventory with a baseline. Changes among the entries that
/// could be read are reported even when coverage is incomplete, but the
/// integrity result is then `Unknown`, never `Match`.
#[must_use]
pub fn verify(baseline: &Manifest, inventory: &Inventory) -> Verdict {
    let observed = Manifest::from_entries(inventory.entries.clone());
    let changes = diff(baseline, &observed);
    let integrity = if !inventory.is_complete() {
        Integrity::Unknown
    } else if observed.digest == baseline.digest {
        Integrity::Match
    } else {
        Integrity::Drift
    };
    Verdict {
        integrity,
        approval: Approval::NotEvaluated,
        profile: Profile::Observe,
        baseline_digest: baseline.digest,
        observed_digest: inventory.is_complete().then_some(observed.digest),
        changes,
        coverage_issues: inventory.issues.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::digest::Sha256Digest;
    use crate::inventory::IssueReason;
    use crate::manifest::{Entry, EntryKind, LogicalPath};

    fn entry(path: &str, bytes: &[u8]) -> Entry {
        Entry {
            path: LogicalPath::new(path.to_owned()).expect("path"),
            kind: EntryKind::Other,
            size: bytes.len() as u64,
            sha256: Sha256Digest::of(bytes),
        }
    }

    fn inventory(entries: Vec<Entry>, issues: Vec<CoverageIssue>) -> Inventory {
        Inventory { entries, issues }
    }

    #[test]
    fn match_is_not_approval() {
        let baseline = Manifest::from_entries(vec![entry("a", b"1")]);
        let verdict = verify(&baseline, &inventory(vec![entry("a", b"1")], vec![]));
        assert_eq!(verdict.integrity, Integrity::Match);
        assert_eq!(verdict.approval, Approval::NotEvaluated);
    }

    #[test]
    fn byte_change_is_drift() {
        let baseline = Manifest::from_entries(vec![entry("a", b"1")]);
        let verdict = verify(&baseline, &inventory(vec![entry("a", b"2")], vec![]));
        assert_eq!(verdict.integrity, Integrity::Drift);
        assert_eq!(verdict.changes.len(), 1);
    }

    #[test]
    fn incomplete_inspection_is_unknown_even_without_visible_change() {
        let baseline = Manifest::from_entries(vec![entry("a", b"1")]);
        let issue = CoverageIssue {
            path: "b".to_owned(),
            reason: IssueReason::Symlink,
        };
        let verdict = verify(&baseline, &inventory(vec![entry("a", b"1")], vec![issue]));
        assert_eq!(verdict.integrity, Integrity::Unknown);
        assert_eq!(verdict.observed_digest, None);
    }
}
