---
title: Modèle de sécurité
description: Ce que Syngard garantit, par profil, et ce qu'il ne garantit pas.
---

## Profils

| Profil | Fournit | Ne garantit pas | État |
| --- | --- | --- | --- |
| **Observe** | Inventaire, dérive constatée, couverture, sortie JSON | L'empêchement de lecture ou d'exécution | Disponible |
| **Preflight** | Vérification au lancement de l'agent via une intégration volontaire | Que le lanceur ne puisse pas être contourné | Prévu |
| **Enforce** | Session gérée, sources approuvées immuables, état protégé | Les processus hors périmètre ; un root hostile | Prévu, à prouver |

Un profil n'est annoncé qu'une fois démontré par des tests adversariaux.

## Propriétés tenues aujourd'hui

- Un contenu modifié n'hérite jamais de l'ancienne conformité : un octet change le digest.
- Le bundle inclut sa composition : ajouter, supprimer ou renommer un fichier est une dérive.
- Analyse et approbation sont des états distincts ; une conformité est signalée
  `approval: not_evaluated`.
- Une inspection incomplète n'est jamais signalée conforme.
- Le scanner n'exécute jamais les fichiers inspectés et ne télécharge jamais les URLs qu'ils contiennent.
- Des noms hostiles ne peuvent pas injecter de séquences terminal dans la sortie texte.

## Menaces considérées

1. Un dépôt ou un skill tiers livre des instructions piégées.
2. Un agent compromis écrit dans son workspace et lance des processus avec ses droits.
3. Un processus local ayant les mêmes droits que l'agent.

L'autorité d'approbation prévue s'exécute sous une identité OS distincte, dont
l'agent ne peut pas écrire l'état. Un démon lancé sous le même utilisateur, ou
un attribut lecture seule appartenant à ce même utilisateur, n'apporte pas cette
séparation.

## Non-objectifs explicites

Syngard ne promet pas de :

- détecter toute intention malveillante ou erreur en langage naturel ;
- résister à un administrateur local ou un noyau hostile ;
- protéger un agent lancé hors du lanceur géré ;
- purger une instruction déjà chargée dans le contexte d'un modèle en restaurant
  le fichier : une telle session doit être considérée contaminée et relancée ;
- couvrir les autres canaux d'injection de prompt, comme les pages web ou les courriels.

Signalez les vulnérabilités en privé : voir la
[politique de sécurité](https://github.com/NTIC-Solution/syngard/security/policy).
