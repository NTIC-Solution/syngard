---
title: Format du manifeste
description: Le format déterministe de baseline, syngard.manifest/v1.
---

Une baseline est un manifeste JSON. Son schéma est publié dans
[`schemas/manifest.v1.schema.json`](https://github.com/NTIC-Solution/syngard/blob/main/schemas/manifest.v1.schema.json).

```json
{
  "digest": "0899d144…00e8",
  "format": "syngard.manifest/v1",
  "entries": [
    {
      "path": "SKILL.md",
      "kind": "instruction",
      "size": 195,
      "sha256": "1791790a…3b7a"
    }
  ]
}
```

## Règles

- `sha256` est le SHA-256 des **octets bruts** du fichier : 64 caractères hexa
  minuscules. Aucune normalisation des fins de ligne, espaces, BOM ou Unicode :
  `\n` et `\r\n` donnent des digests différents.
- `entries` est trié strictement par octets du chemin. `path` est relatif,
  séparé par `/`, sans segment vide, `.` ou `..`, sans antislash.
- `digest` est le SHA-256 de la sérialisation JSON compacte de
  `{"format": …, "entries": […]}`, champs dans l'ordre ci-dessus. Ajouter,
  supprimer, renommer ou modifier une entrée le change.
- Les dates de modification ne font pas partie du manifeste : elles ne prouvent
  pas l'intégrité.

## Chargement

Au chargement d'une baseline, syngard recalcule le digest. Un écart, des entrées
non triées, un `format` inconnu ou un chemin invalide rejettent le fichier
(code 4). Une baseline altérée n'est jamais réparée.
