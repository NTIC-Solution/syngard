---
title: Verify in CI
description: Fail a pipeline when instruction files drift from their baseline.
---

Commit a baseline next to the bundle it describes, then run `verify` in CI. The
job fails on drift (exit 2) and on incomplete inspection (exit 3).

```yaml
# .github/workflows/instructions.yml
name: Instructions
on: [pull_request]
permissions:
  contents: read
jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@<full-commit-sha> # pin by SHA
        with:
          persist-credentials: false
      - name: Build syngard
        run: |
          git clone --depth 1 https://github.com/NTIC-Solution/syngard "$RUNNER_TEMP/syngard"
          cargo build --release --locked --manifest-path "$RUNNER_TEMP/syngard/Cargo.toml"
      - name: Verify skills
        run: |
          "$RUNNER_TEMP/syngard/target/release/syngard" verify .claude/skills/my-skill \
            --baseline .claude/skills/my-skill.baseline.json --format json
```

## Updating the baseline

When an instruction change is intended, regenerate the baseline in the same
pull request and let a reviewer read both diffs:

```sh
rm my-skill.baseline.json
syngard scan ./my-skill --output my-skill.baseline.json
```

:::caution
In CI, the baseline lives in the same repository as the change. A pull request
can modify both, so the review of the baseline diff is the control. Protected
approvals held outside the workspace are on the [roadmap](/syngard/project/roadmap/).
:::

## Pin the version

Clone a tagged release or a specific commit once releases exist, rather than
the moving `main` branch.
