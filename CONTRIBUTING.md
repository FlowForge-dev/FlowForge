# Contributing

Thanks for helping improve FlowForge.

## Local Setup

```bash
git clone https://github.com/AndroRAT-user/FlowForge.git
cd FlowForge
cargo build
cargo test
```

## Before Opening A PR

Run:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

Keep changes focused. If you are changing provider behavior, add or update mocked streaming tests in `src/providers.rs`.

## Project Principles

- Terminal-first always.
- No web UI, Electron, or desktop shell.
- Keep provider integrations behind `ProviderClient`.
- Do not add providers unless there is a strong maintainer-approved reason.
- Prefer small, testable changes.
- Avoid storing secrets in files.

## Commit Style

Use short, plain commit messages:

```text
add provider health checks
fix workflow install path
document ollama setup
```

## Pull Request Checklist

- The change is scoped.
- Tests pass.
- CLI behavior is documented when user-facing.
- Security implications are called out.
- New files are placed in the existing structure.

## Issue Reports

Include:

- OS and shell
- `forge --version`
- command used
- expected result
- actual result
- relevant config with secrets removed
