---
title: JSON reports
description: Machine-readable output of scan, diff and verify.
---

Every command accepts `--format json`. Each report has a closed JSON Schema
(draft 2020-12) in [`schemas/`](https://github.com/NTIC-Solution/syngard/tree/main/schemas),
and the test suite validates real CLI output against them: a new field cannot
appear without a schema change.

| Command | Schema |
| --- | --- |
| `scan` | `scan-report.v1.schema.json` |
| `diff` | `diff-report.v1.schema.json` |
| `verify` | `verify-report.v1.schema.json` |

## `verify` example

```json
{
  "integrity": "drift",
  "approval": "not_evaluated",
  "profile": "observe",
  "baseline_digest": "0899d144…00e8",
  "observed_digest": "846e2aea…1cf1",
  "changes": [
    {
      "change": "modified",
      "before": { "path": "scripts/format.sh", "kind": "script", "size": 91, "sha256": "baf6…4057" },
      "after":  { "path": "scripts/format.sh", "kind": "script", "size": 105, "sha256": "54be…1e24" }
    }
  ],
  "coverage_issues": []
}
```

`observed_digest` is `null` when the inspection is incomplete; `coverage_issues`
then lists each entry with its `reason` (see [CLI reference](/syngard/reference/cli/)).
