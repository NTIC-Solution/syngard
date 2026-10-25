default:
    @just --list

# Install the repository git hooks (secret scan, signing check, formatting).
setup:
    git config core.hooksPath .githooks

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all --check

lint:
    cargo clippy --workspace --all-targets --locked -- -D warnings

test:
    cargo test --workspace --locked

build:
    cargo build --workspace --locked

# Secrets in the full history; needs gitleaks.
secrets:
    gitleaks git --redact --verbose .

# Licenses, advisories and sources; needs cargo-deny.
deny:
    cargo deny check

# Build the documentation site (Node 22+, pnpm via corepack).
docs:
    cd docs && corepack pnpm install --frozen-lockfile && corepack pnpm build

# Serve the documentation locally.
docs-dev:
    cd docs && corepack pnpm dev

check: fmt-check lint test build
