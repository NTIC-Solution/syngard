---
title: Quick start
description: Record a baseline, detect drift, verify in CI.
---

A **bundle** is a directory of instruction files and everything they reference:
`SKILL.md`, scripts, references, configuration.

## 1. Record a baseline

```sh
syngard scan ./my-skill --output baseline.json
```

`scan` prints every file with its SHA-256 and kind, then the manifest digest.
With `--output`, it writes the manifest as a baseline file. It refuses to write
when the inspection is incomplete, and never overwrites an existing file.

:::note
A baseline records what the bundle looked like. It is **not** an approval:
anyone who can edit the bundle can usually edit the baseline too. See
[security model](/syngard/concepts/security-model/).
:::

## 2. See what changed

```sh
syngard diff ./my-skill --against baseline.json
```

```text
~ scripts/format.sh  baf6764e…4057 -> 54be6698…1e24
+ AGENTS.md
- references/old.md
```

`+` added, `-` removed, `~` modified (old and new digests).

## 3. Verify

```sh
syngard verify ./my-skill --baseline baseline.json
echo $?
```

```text
integrity: drift
approval:  not evaluated (baseline is not an approval)
profile:   observe
```

| Exit code | Meaning |
| :-: | --- |
| `0` | Complete inspection, no drift |
| `2` | Drift: changed, added or removed file |
| `3` | Incomplete inspection: never compliant |
| `4` | Usage error, or invalid or tampered baseline |

Add `--format json` to any command for machine-readable output. Next:
[use it in CI](/syngard/guides/ci/).
