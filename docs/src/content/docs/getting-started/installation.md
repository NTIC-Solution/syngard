---
title: Installation
description: Build syngard from source.
---

No binary releases are published yet. Build from source with the Rust toolchain
pinned by the repository.

## Requirements

- Linux (other platforms are untested; see [the roadmap](/syngard/project/roadmap/)).
- [Rust](https://rustup.rs/) via rustup. The exact toolchain is read from
  `rust-toolchain.toml`; the minimum supported version is 1.88.
- Git.

## Build

```sh
git clone https://github.com/NTIC-Solution/syngard
cd syngard
cargo build --release --locked
./target/release/syngard --version
```

`--locked` refuses to build if `Cargo.lock` would change, so you build exactly
the dependency versions that were reviewed.

## Verify the source

Every commit on `main` is SSH-signed. To check signatures locally:

```sh
git config gpg.ssh.allowedSignersFile .github/allowed_signers
git log --show-signature -1
```
