---
title: Démarrage rapide
description: Enregistrer une baseline, détecter une dérive, vérifier en CI.
---

Un **bundle** est un répertoire de fichiers d'instructions et de tout ce qu'ils
référencent : `SKILL.md`, scripts, références, configuration.

## 1. Enregistrer une baseline

```sh
syngard scan ./my-skill --output baseline.json
```

`scan` affiche chaque fichier avec son SHA-256 et son type, puis le digest du
manifeste. Avec `--output`, il écrit le manifeste comme baseline. Il refuse
d'écrire si l'inspection est incomplète et n'écrase jamais un fichier existant.

:::note
Une baseline enregistre l'état du bundle. Ce n'est **pas** une approbation :
quiconque peut modifier le bundle peut en général modifier la baseline aussi.
Voir le [modèle de sécurité](/syngard/fr/concepts/security-model/).
:::

## 2. Voir ce qui a changé

```sh
syngard diff ./my-skill --against baseline.json
```

```text
~ scripts/format.sh  baf6764e…4057 -> 54be6698…1e24
+ AGENTS.md
- references/old.md
```

`+` ajouté, `-` supprimé, `~` modifié (ancien et nouveau digest).

## 3. Vérifier

```sh
syngard verify ./my-skill --baseline baseline.json
echo $?
```

```text
integrity: drift
approval:  not evaluated (baseline is not an approval)
profile:   observe
```

| Code de sortie | Sens |
| :-: | --- |
| `0` | Inspection complète, aucune dérive |
| `2` | Dérive : fichier modifié, ajouté ou supprimé |
| `3` | Inspection incomplète : jamais conforme |
| `4` | Erreur d'utilisation, ou baseline invalide ou altérée |

Ajoutez `--format json` à toute commande pour une sortie exploitable par machine.
Ensuite : [l'utiliser en CI](/syngard/fr/guides/ci/).
