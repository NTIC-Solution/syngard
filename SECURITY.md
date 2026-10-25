# Security policy

## Reporting a vulnerability

Do not open a public issue. Use
[GitHub private vulnerability reporting](https://github.com/NTIC-Solution/syngard/security/advisories/new)
or email **support@nticsolution.com**.

Include the version or commit, operating system, reproduction steps and observed
impact. Use inert fixtures, reserved domains (`example.invalid`) and fake
secrets. We acknowledge reports within 5 business days and publish a GitHub
advisory once a fix is available, with credit if you wish.

## Supported versions

| Version | Supported |
| --- | --- |
| `main` (pre-alpha, observe profile) | Best effort |
| Releases | None published yet |

## Trust model in brief

- Every inspected file is treated as hostile: it is never executed, URLs it
  mentions are never fetched, and symlinks are never followed.
- An incomplete inspection is never reported as compliant.
- A baseline is an ordinary file the user (and therefore an agent) can edit. It
  detects drift; it never authorizes anything. Approval will come from a
  separate authority running under a distinct OS identity.
- A local control does not claim to resist a hostile administrator or kernel.

Details: [security model](https://ntic-solution.github.io/syngard/concepts/security-model/).

## Secrets

No secrets in the repository, fixtures, logs or issues, ever. gitleaks blocks
commits locally (`just setup`) and scans the full history in CI; GitHub secret
scanning with push protection is enabled. An exposed secret is considered
compromised: revoke and rotate it first, then assess the history. Deleting the
commit is not enough.

## Repository integrity

Every commit on `main` is SSH-signed by a maintainer listed in
[`.github/allowed_signers`](.github/allowed_signers); the `Commits` workflow
verifies this. GitHub Actions are pinned to full commit SHAs.
