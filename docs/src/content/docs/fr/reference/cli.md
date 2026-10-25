---
title: Référence CLI
description: Commandes, options et codes de sortie.
---

```text
syngard scan   <root> [--format text|json] [--output <file>]
syngard diff   <root> --against <baseline> [--format text|json]
syngard verify <root> --baseline <baseline> [--format text|json]
```

`<root>` doit être un vrai répertoire. Une racine symlink est refusée (code 4).

## `scan`

Inventorie chaque fichier ordinaire sous `<root>` et affiche son digest, son type
et son chemin, puis le digest du manifeste.

- `--output <file>` écrit le manifeste comme baseline. Refusé si l'inspection est
  incomplète ; n'écrase jamais un fichier existant.
- Code 0 si complète, 3 si incomplète.

## `diff`

Compare le bundle à une baseline et liste les entrées ajoutées (`+`), supprimées
(`-`) et modifiées (`~`).

- Code 0 sans changement, 2 avec changements, 3 si incomplète.

## `verify`

Même comparaison, présentée comme une décision à dimensions séparées :

| Champ | Valeurs |
| --- | --- |
| `integrity` | `match`, `drift`, `unknown` |
| `approval` | `not_evaluated` (aucune autorité d'approbation n'existe encore) |
| `profile` | `observe` |

- Code 0 sur `match`, 2 sur `drift`, 3 sur `unknown`.

## Codes de sortie

| Code | Sens |
| :-: | --- |
| `0` | Inspection complète, politique satisfaite |
| `2` | Violation de politique (dérive) |
| `3` | Inspection incomplète : un élément n'a pas pu être inspecté |
| `4` | Erreur d'utilisation, baseline illisible ou invalide, racine qui n'est pas un répertoire |

`3` l'emporte toujours sur `0` : une inspection incomplète n'est jamais conforme.

## Ce qui rend une inspection incomplète

| Raison | Cause |
| --- | --- |
| `symlink` | Les symlinks ne sont jamais suivis |
| `hardlink` | Un fichier à plusieurs liens peut changer par un autre chemin |
| `special_file` | FIFO, socket ou périphérique ; jamais ouvert |
| `non_utf8_path` | Nom de fichier non UTF-8 |
| `too_large` | Fichier de plus de 8 Mio |
| `too_deep` | Imbrication de plus de 32 niveaux |
| `too_many_files` | Plus de 10 000 fichiers |
| `changed_during_read` | Le fichier a dépassé la limite pendant la lecture |
| `io` | Permission refusée ou autre erreur d'E/S |

## Types de fichiers

La classification est informative ; l'intégrité couvre chaque fichier, quel que
soit son type.

| Type | Règle |
| --- | --- |
| `instruction` | Exactement `AGENTS.md`, `CLAUDE.md` ou `SKILL.md` (casse respectée) |
| `script` | `.sh .bash .zsh .py .js .mjs .ts .rb .pl .ps1`, ou bit exécutable |
| `reference` | `.md`, `.txt` |
| `config` | `.json .toml .yaml .yml` |
| `other` | Tout le reste |
