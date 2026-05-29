# Release Readiness Report

ForgeFlow is a strong prototype with a useful terminal-first thesis, but it is not ready for a high-trust public launch without hardening. The core risk is not missing features. The risk is that a developer installs a tool that can call remote AI APIs, run shell commands, write files, discover plugins, and store memory before the safety boundaries, packaging story, and contributor contract are mature.

## Scores

| Area | Score |
| --- | ---: |
| Architecture | 7/10 |
| Security | 4/10 |
| Maintainability | 6/10 |
| Extensibility | 7/10 |
| User Experience | 6/10 |
| Documentation | 7/10 |
| Testing | 4/10 |
| Open Source Readiness | 6/10 |

Overall: 5.9/10. Promising, demoable, but not yet boring enough for trust.

## Issues

### 1. Arbitrary Shell Execution

Severity: Critical

Why it matters: `forge tool shell` executes any command passed by the user. That is useful for power users, but dangerous when combined with AI-generated workflows, plugins, copied README snippets, or marketplace content.

Concrete fix: Add an explicit danger warning in docs, require an opt-in config flag for shell execution, and block shell execution from workflows/plugins unless explicitly allowed.

Effort: Medium

### 2. Untrusted Plugin Execution Path

Severity: Critical

Why it matters: The plugin system scaffolds `bin/` and advertises command/tool extensibility, but there is no trust model, signing, sandboxing, permissions manifest, or install warning.

Concrete fix: Define plugin permissions, require manifest validation before install, mark plugins as untrusted by default, and document that plugins are executable code.

Effort: Large

### 3. File Write Tool Can Overwrite Arbitrary Paths

Severity: High

Why it matters: `forge tool write` writes to any user-supplied path. That can overwrite project files, shell profiles, or sensitive local files.

Concrete fix: Require confirmation for writes outside the current repository or `~/.forgeflow`, add `--force` for noninteractive usage, and add path normalization checks.

Effort: Medium

### 4. Plugin Remove Deletes Directories Recursively

Severity: High

Why it matters: `remove_dir_all` is called on a computed plugin path. If plugin names are not constrained, path traversal or confusing names could delete unexpected directories.

Concrete fix: Validate plugin names against a strict slug regex and verify canonical paths remain inside `plugins_dir`.

Effort: Small

### 5. API Keys May Be Stored In Plain TOML

Severity: High

Why it matters: Config supports direct `api_key` values. Users will paste secrets into files and accidentally commit or share them.

Concrete fix: Prefer `api_key_env`, warn when direct secrets are detected, add `forge config doctor`, and document env-var setup first.

Effort: Small

### 6. Provider Streaming Parser Is Hand-Rolled

Severity: High

Why it matters: SSE parsing is subtle. Multi-line `data:` events, provider-specific error events, partial UTF-8, and stream termination edge cases can corrupt output or hide failures.

Concrete fix: Add a small tested SSE parser abstraction or use a proven SSE crate. Expand tests for chunk boundaries, multi-line events, provider errors, and malformed JSON.

Effort: Medium

### 7. Retry Logic May Duplicate Expensive Requests

Severity: Medium

Why it matters: Retrying completion requests can duplicate token spend and produce inconsistent results. Some providers may have already generated output before a connection drop.

Concrete fix: Retry only before any stream delta is received, document retry semantics, and expose retry count in provider info.

Effort: Medium

### 8. No Rate Limit Or Cost Visibility

Severity: Medium

Why it matters: Users can run chains, workflows, and agent pipelines that make many model calls with no estimate or warning.

Concrete fix: Add dry-run call counts, token-budget config, and clear docs explaining that workflows and agents may perform multiple paid calls.

Effort: Medium

### 9. Chat Stores Transcript In Memory Only

Severity: Low

Why it matters: Long chat sessions grow unbounded and may hit context or process memory limits.

Concrete fix: Add transcript truncation/summarization before sending and document chat limitations.

Effort: Medium

### 10. Project Index Reads Entire Files Into Memory

Severity: Medium

Why it matters: Large repositories, generated files, binaries, or vendored code can cause slow scans and high memory usage.

Concrete fix: Add max file size, binary detection, ignored directory defaults, and indexing stats.

Effort: Medium

### 11. Search Reads Every File As UTF-8 Text

Severity: Medium

Why it matters: `forge tool search` may waste time reading binary files and large artifacts.

Concrete fix: Use streaming line reads, binary detection, and file size caps. Prefer `ignore` plus content type guards.

Effort: Small

### 12. Workflow Step Model Is Too Loose

Severity: Medium

Why it matters: Workflows are plain strings mapped to patterns. That is simple, but it prevents static validation and makes future tool/agent steps ambiguous.

Concrete fix: Introduce typed workflow steps while continuing to accept the legacy string format.

Effort: Medium

### 13. Agent Outputs Are Plain Text

Severity: Medium

Why it matters: Agents pass blobs of text, so the Coder, Tester, and Reviewer cannot reliably consume structured plans, file changes, or test commands.

Concrete fix: Define structured agent outputs with `summary`, `artifacts`, `commands`, `risks`, and `next_steps`.

Effort: Large

### 14. No End-To-End CLI Tests

Severity: High

Why it matters: Unit tests cover provider parsers, but not real CLI behavior, config loading, stdin piping, workflow execution, memory persistence, or plugin commands.

Concrete fix: Add integration tests using `assert_cmd`, temp `FORGEFLOW_HOME`, and mocked providers.

Effort: Medium

### 15. No Release Binaries

Severity: High

Why it matters: `cargo install --path .` is fine for Rust developers, but most CLI users expect prebuilt binaries or `cargo install forgeflow`.

Concrete fix: Add GitHub release workflow using `cargo-dist` or `cross`, publish checksums, and document install paths.

Effort: Medium

### 16. Crate And Repo Naming Are Confusing

Severity: Medium

Why it matters: Repository is `FlowForge`, package is `forgeflow`, binary is `forge`, and docs use both FlowForge and ForgeFlow. Users may not know what to search, install, or star.

Concrete fix: Decide the public brand hierarchy and put it in the README: "FlowForge is the repo, ForgeFlow is the CLI." Consider aligning crate/repo names before publishing.

Effort: Small

### 17. No Threat Model

Severity: High

Why it matters: This tool touches secrets, files, shell commands, plugins, HTTP, memory, and AI output. That needs an explicit threat model before trust-sensitive users adopt it.

Concrete fix: Add `docs/THREAT_MODEL.md` covering assets, trust boundaries, attacker capabilities, and mitigations.

Effort: Medium

### 18. No Public Roadmap Or Maintainer Policy

Severity: Medium

Why it matters: Open-source users need to know whether this is stable, experimental, abandoned, or accepting contributions.

Concrete fix: Add `ROADMAP.md`, issue labels, governance expectations, and release cadence.

Effort: Small

### 19. No MSRV Policy

Severity: Low

Why it matters: Rust users care which compiler version is supported. The project currently uses edition 2024 and recent dependencies.

Concrete fix: State MSRV in README and CI.

Effort: Small

### 20. Provider Defaults May Not Exist

Severity: Medium

Why it matters: `gpt-5.5` may not be available to all users. A first-run failure caused by a fantasy/default unavailable model looks amateur.

Concrete fix: Use conservative known model defaults per provider and document how to override them.

Effort: Small

### 21. No Config Doctor

Severity: Medium

Why it matters: Users will struggle with missing API keys, bad Ollama hosts, bad models, and invalid TOML.

Concrete fix: Add `forge config doctor` later. For this release, improve README troubleshooting.

Effort: Medium

### 22. No Man Pages Or Shell Completions

Severity: Low

Why it matters: Terminal-first users expect completions and reference docs.

Concrete fix: Generate clap completions and man pages during release.

Effort: Small

### 23. Provider Test Can Spend Money

Severity: Medium

Why it matters: `forge provider test` makes live model calls for configured remote providers. Users should understand it may use paid credits.

Concrete fix: Print a cost warning before testing all configured remote providers, and support `--dry-run`.

Effort: Small

### 24. Memory Has No Privacy Controls

Severity: High

Why it matters: Persistent memory may store sensitive project notes or model outputs in SQLite without encryption, expiration, or easy export.

Concrete fix: Document storage location clearly, add `memory clear` warnings, and later add encryption/export/retention settings.

Effort: Medium

### 25. Documentation Lacks Failure Modes

Severity: Medium

Why it matters: Docs show happy paths but not bad API keys, rate limits, Ollama not running, invalid patterns, or workflow failures.

Concrete fix: Add troubleshooting sections for every first-run failure.

Effort: Small

## Release Plan

### Pre-Release Checklist

- Run `cargo fmt --check`.
- Run `cargo clippy -- -D warnings`.
- Run `cargo test`.
- Add CLI integration tests.
- Review all commands that write files or execute shell commands.
- Confirm provider docs against current vendor APIs.
- Pick stable default models.
- Add threat model.
- Add release binaries and checksums.
- Verify README from a clean machine.
- Tag `v0.1.0`.

### GitHub Repository Structure

```text
.github/
  workflows/ci.yml
  dependabot.yml
docs/
  ARCHITECTURE.md
  RELEASE_READINESS_REPORT.md
examples/
  patterns/
  workflows/
src/
  agents.rs
  cli.rs
  config.rs
  memory.rs
  patterns.rs
  plugins.rs
  project.rs
  providers.rs
  terminal.rs
  tools.rs
  workflows.rs
Cargo.toml
Cargo.lock
README.md
LICENSE
CONTRIBUTING.md
CODE_OF_CONDUCT.md
SECURITY.md
```

### Recommended License

Use MIT for the first public release. It is simple, permissive, familiar, and low-friction for CLI adoption. Dual MIT/Apache-2.0 is also common in Rust, but this repo now includes MIT for clarity.

### CONTRIBUTING.md

Included at repository root. It covers local setup, checks, project principles, commit style, PR checklist, and issue reports.

### CODE_OF_CONDUCT.md

Included at repository root. It establishes basic participation rules and maintainer enforcement.

### SECURITY.md

Included at repository root. It explains private vulnerability reporting, supported versions, and user security expectations.

### GitHub Actions CI/CD

Included at `.github/workflows/ci.yml`:

- checkout
- Rust toolchain
- cargo cache
- format check
- clippy with warnings denied
- test

Next release workflow should add:

- tagged release build
- Linux/macOS/Windows artifacts
- checksums
- SBOM
- cargo publish dry run

### Release Workflow

1. Merge only green CI.
2. Update `CHANGELOG.md`.
3. Run release checks locally.
4. Tag `v0.1.0`.
5. Build artifacts.
6. Publish GitHub release.
7. Announce with honest limitations.

### Versioning Strategy

Use SemVer:

- `0.1.x`: experimental public preview
- `0.2.x`: breaking workflow/plugin schema changes allowed
- `1.0.0`: stable CLI contracts, config format, plugin manifest, and provider behavior

## Brutal Assessment

This section uses audience lenses, not impersonation.

### NetworkChuck Audience

Why they would install it: It has the magic terminal demo shape: bring an API key, run a command, watch AI do useful work. `forge agent debug-project` is a good video hook.

Why they would ignore it: Setup still smells like a Rust dev project, not a one-command install. No binaries means casual users bounce.

What would make them star it: A clean "install, configure Ollama, debug a broken repo in 60 seconds" demo.

What would make them uninstall it: API key confusion, provider errors, or a scary shell/file command moment.

### ThePrimeagen Audience

Why they would install it: Terminal-first, Rust, no Electron, no web UI. That earns attention.

Why they would ignore it: If agent output is just vibes and markdown, they will call it wrapperware.

What would make them star it: Fast project indexing, transparent commands, and agents that produce verifiable diffs/tests.

What would make them uninstall it: Slow scans, noisy output, hidden API calls, or weak defaults.

### Fireship Audience

Why they would install it: The concept is easy to explain in 100 seconds: "n8n plus Fabric for your terminal."

Why they would ignore it: Too many commands without a killer first-run path.

What would make them star it: A tiny, polished demo where one command turns logs into a fix plan and review.

What would make them uninstall it: Long setup, missing binaries, or docs that require reading architecture first.

### Random Rust Developer

Why they would install it: It is Rust, hackable, and the module boundaries are understandable.

Why they would ignore it: The safety story is undercooked for a tool that runs shell commands and plugins.

What would make them star it: Strong tests, clear trait boundaries, real provider mocks, and no JS runtime.

What would make them uninstall it: Clippy warnings, panics, weird model defaults, or unsafe file behavior.

## Launch Verdict

Do not market this as stable yet. Launch it as `v0.1.0 experimental`, be blunt about shell/plugin risks, and make the first-run path excellent. The product idea is strong. The trust layer needs to catch up.
