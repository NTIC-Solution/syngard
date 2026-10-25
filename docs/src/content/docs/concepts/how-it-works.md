---
title: How it works
description: Inventory, manifest, comparison.
---

Coding agents act on plain text: `AGENTS.md`, `CLAUDE.md`, `SKILL.md`, and the
scripts and references those files point to. A new file in a parent directory, a
script edited next to a skill that was already reviewed, an instruction dropped
into a downloads folder: the agent's behavior changes and nobody approved it.

Syngard answers one question: **are the bytes my agents will read exactly the
ones that were recorded for this bundle?**

## Three separate decisions

- **Analysis** explains risk in content (planned: deterministic rules).
- **Integrity** establishes whether content is byte-identical to a reference.
- **Approval and enforcement** decide whether a session may use it (planned:
  separate authority).

These are never merged into a single "safe" badge. A file without findings is
not approved; a file that matches its baseline is not approved either.

## Pipeline

1. **Inventory.** Walk the bundle without following symlinks. Read each regular
   file with a size limit. Anything that cannot be inspected becomes a coverage
   issue.
2. **Manifest.** Hash raw bytes, sort entries, compute the manifest digest.
3. **Compare.** Diff against the baseline entry by entry, then decide:
   `match`, `drift`, or `unknown` when coverage is incomplete.

Nothing is executed, no network access happens, and every output escapes
control characters from file names.
