# Contributing

**Remember:** small PRs are good PRs.

## Setup

Fast path:

```bash
cargo install --git https://github.com/FlowForge-dev/FlowForge
```

Repo path:

```bash
git clone https://github.com/FlowForge-dev/FlowForge.git
cd FlowForge
cargo build
cargo test
```

## Before You Push

Run all three:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## Project Rules

- Terminal-first.
- No web UI.
- No Electron.
- Keep providers behind `ProviderClient`.
- Do not add random new providers.
- Do not log secrets.
- Keep changes small.

## Good First PRs

- Fix docs.
- Add tests.
- Improve errors.
- Improve examples.
- Simplify setup.

## PR Checklist

- The change is easy to explain.
- Tests pass.
- Docs changed if commands changed.
- Security impact is mentioned.

## Bug Reports

Please include:

- OS
- shell
- `forge --version`
- command used
- expected result
- actual result
- config with secrets removed
