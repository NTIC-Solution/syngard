---
title: Fonctionnement
description: Inventaire, manifeste, comparaison.
---

Les agents de code agissent sur du texte : `AGENTS.md`, `CLAUDE.md`, `SKILL.md`,
et les scripts et références vers lesquels ces fichiers pointent. Un nouveau
fichier dans un dossier parent, un script modifié à côté d'un skill déjà relu,
une instruction déposée dans un dossier de téléchargements : le comportement de
l'agent change sans que personne l'ait approuvé.

Syngard répond à une question : **les octets que mes agents vont lire sont-ils
exactement ceux qui ont été enregistrés pour ce bundle ?**

## Trois décisions séparées

- **L'analyse** explique le risque dans le contenu (prévu : règles déterministes).
- **L'intégrité** établit si le contenu est identique, octet pour octet, à une référence.
- **L'approbation et l'application** décident si une session peut l'utiliser
  (prévu : autorité séparée).

Elles ne sont jamais fusionnées en un badge « sûr ». Un fichier sans alerte n'est
pas approuvé ; un fichier conforme à sa baseline ne l'est pas non plus.

## Chaîne de traitement

1. **Inventaire.** Parcourir le bundle sans suivre les symlinks. Lire chaque
   fichier ordinaire avec une limite de taille. Tout ce qui ne peut pas être
   inspecté devient un problème de couverture.
2. **Manifeste.** Hacher les octets bruts, trier les entrées, calculer le digest.
3. **Comparaison.** Comparer à la baseline entrée par entrée, puis décider :
   `match`, `drift`, ou `unknown` quand la couverture est incomplète.

Rien n'est exécuté, aucun accès réseau n'a lieu, et toute sortie échappe les
caractères de contrôle des noms de fichiers.
