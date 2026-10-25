---
title: Security model
description: What Syngard guarantees, per profile, and what it does not.
---

## Profiles

| Profile | Delivers | Does not guarantee | Status |
| --- | --- | --- | --- |
| **Observe** | Inventory, detected drift, coverage, JSON output | Preventing reads or execution | Available |
| **Preflight** | Check at agent launch through an opt-in integration | That the launcher cannot be bypassed | Planned |
| **Enforce** | Managed session, immutable approved sources, protected state | Processes outside the managed scope; hostile root | Planned, to be proven |

A profile is announced only once adversarial tests demonstrate it.

## Properties held today

- A content change never inherits the old match: one byte changes the digest.
- The bundle includes its composition: adding, removing or renaming a file is drift.
- Analysis and approval are separate states; a match is reported as
  `approval: not_evaluated`.
- An incomplete inspection is never reported as compliant.
- The scanner never executes inspected files and never fetches URLs they contain.
- Hostile names cannot inject terminal sequences into text output.

## Threats considered

1. A third-party repository or skill ships booby-trapped instructions.
2. A compromised agent writes in its workspace and starts processes with its rights.
3. A local process with the same rights as the agent.

The planned approval authority runs under a distinct OS identity whose state the
agent cannot write. A daemon running as the same user, or a read-only flag the
same user owns, does not provide that separation.

## Explicit non-goals

Syngard does not promise to:

- detect every malicious intent or error in natural language;
- resist a hostile local administrator or kernel;
- protect an agent started outside the managed launcher;
- purge an instruction already loaded into a model's context by restoring the
  file: such a session must be treated as contaminated and restarted;
- cover other prompt-injection channels such as web pages or email.

Report vulnerabilities privately: see the
[security policy](https://github.com/NTIC-Solution/syngard/security/policy).
