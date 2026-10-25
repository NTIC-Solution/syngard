---
title: Vérifier en CI
description: Faire échouer un pipeline quand des fichiers d'instructions dérivent.
---

Versionnez une baseline à côté du bundle qu'elle décrit, puis lancez `verify` en
CI. Le job échoue sur une dérive (code 2) et sur une inspection incomplète (code 3).

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
      - uses: actions/checkout@<sha-complet-du-commit> # épingler par SHA
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

## Mettre à jour la baseline

Quand un changement d'instruction est voulu, régénérez la baseline dans la même
pull request et laissez un relecteur lire les deux diffs :

```sh
rm my-skill.baseline.json
syngard scan ./my-skill --output my-skill.baseline.json
```

:::caution
En CI, la baseline vit dans le même dépôt que le changement. Une pull request
peut modifier les deux : la relecture du diff de baseline est le contrôle. Des
approbations protégées hors du workspace sont prévues dans la
[feuille de route](/syngard/fr/project/roadmap/).
:::

## Épingler la version

Clonez une release taguée ou un commit précis dès que des releases existent,
plutôt que la branche `main`, qui bouge.
