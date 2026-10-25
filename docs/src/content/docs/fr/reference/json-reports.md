---
title: Rapports JSON
description: Sortie exploitable par machine de scan, diff et verify.
---

Chaque commande accepte `--format json`. Chaque rapport a un JSON Schema fermé
(draft 2020-12) dans [`schemas/`](https://github.com/NTIC-Solution/syngard/tree/main/schemas),
et la suite de tests valide la sortie réelle de la CLI contre eux : un nouveau
champ ne peut pas apparaître sans modification du schéma.

| Commande | Schéma |
| --- | --- |
| `scan` | `scan-report.v1.schema.json` |
| `diff` | `diff-report.v1.schema.json` |
| `verify` | `verify-report.v1.schema.json` |

## Exemple `verify`

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

`observed_digest` vaut `null` quand l'inspection est incomplète ;
`coverage_issues` liste alors chaque entrée avec sa `reason` (voir la
[référence CLI](/syngard/fr/reference/cli/)).
