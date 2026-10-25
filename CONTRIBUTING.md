# Contributing

Thanks for your interest. Syngard is a security tool: every change must come
with its evidence.

## Non-negotiable rules

1. **No secrets**, real or realistic, anywhere. Fixtures use obviously fake
   values and `example.invalid`.
2. **Signed commits from the maintainers.** The `Commits` workflow rejects any
   commit whose author or signing key is not listed in
   `.github/allowed_signers`. External contributions are therefore reviewed in
   their pull request, then re-applied by a maintainer under their signature,
   with credit in the commit message (`Suggested-by:` or `Reported-by:`).
   Dependabot updates follow the same path.
3. **Security properties never regress to make a test pass.** Integrity,
   analysis, approval and enforcement stay separate; an incomplete inspection is
   never success; inspected content is never executed or fetched.

## Setup

```sh
# Rust (exact version from rust-toolchain.toml), just, gitleaks
just setup     # enable .githooks: gitleaks, mandatory signing, rustfmt
just check     # fmt-check, clippy -D warnings, tests, build
just secrets   # gitleaks over the full history
just deny      # licenses, advisories, sources (cargo-deny)
just docs      # build the documentation site (Node 22+, pnpm via corepack)
```

SSH commit signing:

```sh
git config commit.gpgsign true
git config gpg.format ssh
git config user.signingkey ~/.ssh/<your-key>.pub
```

## Making a change

1. Open or pick an issue describing the expected behavior, its milestone and
   acceptance criteria.
2. One branch per issue: `<type>/<number>-<topic>`, e.g. `feat/12-rules-engine`.
3. [Conventional Commits](https://www.conventionalcommits.org/): `feat(core): …`,
   `fix(cli): …`, `docs: …`, `ci: …`.
4. Tests cover the nominal path, invalid input, boundaries and failure. A bug fix
   starts with a failing test.
5. User-facing behavior changes update the documentation in `docs/`.

## Pull request acceptance

- `just check` passes; the Security, Docs and Commits workflows are green.
- No deleted tests or relaxed lints without a written reason.
- New dependencies are justified in the pull request: license, maintenance, and
  a release published at least 14 days earlier.
- Documentation promises nothing beyond what is tested.

## Merging into `main`

`main` is protected by a ruleset: signed commits, linear history, no force push
or deletion, required checks, and no bypass, including for administrators.

GitHub's merge buttons recreate commits with another signature or none, so they
are not used. A maintainer fast-forwards `main` once CI is green on the branch:

```sh
git fetch origin
git rebase origin/main          # commits re-signed locally
git push origin <branch>        # CI runs on these exact SHAs
# once CI is green:
git push origin <branch>:main   # fast-forward; GitHub marks the PR merged
```

## Code of conduct

This project follows the [Contributor Covenant 2.1](CODE_OF_CONDUCT.md).
