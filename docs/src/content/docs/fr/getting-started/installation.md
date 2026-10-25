---
title: Installation
description: Compiler syngard depuis les sources.
---

Aucun binaire n'est encore publié. Compilez depuis les sources avec la chaîne
Rust épinglée par le dépôt.

## Prérequis

- Linux (les autres plateformes ne sont pas testées ; voir [la feuille de route](/syngard/fr/project/roadmap/)).
- [Rust](https://rustup.rs/) via rustup. La version exacte est lue dans
  `rust-toolchain.toml` ; la version minimale supportée est 1.88.
- Git.

## Compiler

```sh
git clone https://github.com/NTIC-Solution/syngard
cd syngard
cargo build --release --locked
./target/release/syngard --version
```

`--locked` refuse de compiler si `Cargo.lock` devait changer : vous compilez
exactement les versions de dépendances qui ont été revues.

## Vérifier les sources

Chaque commit de `main` est signé en SSH. Pour vérifier localement :

```sh
git config gpg.ssh.allowedSignersFile .github/allowed_signers
git log --show-signature -1
```
