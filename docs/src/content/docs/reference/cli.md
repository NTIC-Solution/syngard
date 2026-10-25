---
title: CLI reference
description: Commands, options and exit codes.
---

```text
syngard scan   <root> [--format text|json] [--output <file>]
syngard diff   <root> --against <baseline> [--format text|json]
syngard verify <root> --baseline <baseline> [--format text|json]
```

`<root>` must be a real directory. A symlinked root is refused (exit 4).

## `scan`

Inventories every regular file under `<root>` and prints its digest, kind and
path, followed by the manifest digest.

- `--output <file>` writes the manifest as a baseline. Refused when the
  inspection is incomplete; never overwrites an existing file.
- Exit 0 when complete, 3 when incomplete.

## `diff`

Compares the bundle with a baseline and lists added (`+`), removed (`-`) and
modified (`~`) entries.

- Exit 0 without changes, 2 with changes, 3 when incomplete.

## `verify`

Same comparison, reported as a decision with separate dimensions:

| Field | Values |
| --- | --- |
| `integrity` | `match`, `drift`, `unknown` |
| `approval` | `not_evaluated` (no approval authority exists yet) |
| `profile` | `observe` |

- Exit 0 on `match`, 2 on `drift`, 3 on `unknown`.

## Exit codes

| Code | Meaning |
| :-: | --- |
| `0` | Complete inspection, policy satisfied |
| `2` | Policy violation (drift) |
| `3` | Incomplete inspection: something could not be inspected |
| `4` | Usage error, unreadable or invalid baseline, root not a directory |

`3` always wins over `0`: an incomplete inspection is never compliant.

## What makes an inspection incomplete

| Reason | Cause |
| --- | --- |
| `symlink` | Symlinks are never followed |
| `hardlink` | A file with more than one link could change through another path |
| `special_file` | FIFO, socket or device; never opened |
| `non_utf8_path` | File name is not valid UTF-8 |
| `too_large` | File exceeds 8 MiB |
| `too_deep` | Nesting exceeds 32 levels |
| `too_many_files` | More than 10 000 files |
| `changed_during_read` | File grew past the limit while being read |
| `io` | Permission denied or another I/O error |

## File kinds

Classification is informational; integrity covers every file regardless of kind.

| Kind | Rule |
| --- | --- |
| `instruction` | Exactly `AGENTS.md`, `CLAUDE.md` or `SKILL.md` (case-sensitive) |
| `script` | `.sh .bash .zsh .py .js .mjs .ts .rb .pl .ps1`, or executable bit |
| `reference` | `.md`, `.txt` |
| `config` | `.json .toml .yaml .yml` |
| `other` | Anything else |
