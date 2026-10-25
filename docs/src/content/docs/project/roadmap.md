---
title: Roadmap
description: Milestones and their exit criteria.
---

No dates are committed before the Enforce prototype: closing every path an agent
can use to read instructions is what determines difficulty. Progress is tracked
in [GitHub milestones](https://github.com/NTIC-Solution/syngard/milestones).

| Milestone | Deliverable | Exit criterion |
| --- | --- | --- |
| M1 | `scan` / `diff` / `verify`, manifest, minimal rule engine | Byte or dependency change detected; deterministic output |
| M2 | Protected approvals, revocation, service under a separate identity, audit | The agent identity can neither approve nor alter trust state |
| M3 | File watching, reconciliation, health | Lost events and restarts trigger re-verification, never implicit compliance |
| M4 | Instruction resolvers for coding agents, Linux Enforce | Replacement after verification refused; otherwise the profile stays Preflight |
| M5 | TypeScript SDK, read-only console, CI integration, SARIF/JSONL exports | Hostile rendering executes nothing |
| M6 | Hardening, packaging, public inert corpus | Signed releases with SBOM and provenance; limits published |

Windows and macOS start in observe mode; each native blocking mechanism gets
its own tests before any equivalence is claimed.
