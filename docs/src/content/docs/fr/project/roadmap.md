---
title: Feuille de route
description: Jalons et critères de sortie.
---

Aucune date n'est engagée avant le prototype Enforce : c'est la fermeture de tous
les chemins par lesquels un agent lit des instructions qui détermine la
difficulté. L'avancement est suivi dans les
[milestones GitHub](https://github.com/NTIC-Solution/syngard/milestones).

| Jalon | Livrable | Critère de sortie |
| --- | --- | --- |
| M1 | `scan` / `diff` / `verify`, manifeste, moteur de règles minimal | Changement d'octet ou de dépendance détecté ; sortie déterministe |
| M2 | Approbations protégées, révocation, service sous identité séparée, audit | L'identité de l'agent ne peut ni approuver ni modifier l'état de confiance |
| M3 | Surveillance des fichiers, réconciliation, santé | Événements perdus et redémarrages déclenchent une revérification, jamais une conformité implicite |
| M4 | Résolveurs d'instructions des agents de code, Enforce Linux | Remplacement après vérification refusé ; sinon le profil reste Preflight |
| M5 | SDK TypeScript, console en lecture seule, intégration CI, exports SARIF/JSONL | Le rendu hostile n'exécute rien |
| M6 | Durcissement, packaging, corpus public inerte | Releases signées avec SBOM et provenance ; limites publiées |

Windows et macOS commencent en mode observe ; chaque mécanisme de blocage natif
a ses propres tests avant toute équivalence annoncée.
