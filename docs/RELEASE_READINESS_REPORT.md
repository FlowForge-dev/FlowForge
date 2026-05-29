# Release Readiness Report

FlowForge is now in a credible public-preview launch posture. The original prototype had the right product shape, but several trust issues were too sharp for public release. This pass focused on hardening the existing product rather than adding new provider or UI surface area.

## Scores

| Area | Score |
| --- | ---: |
| Architecture | 9/10 |
| Security | 8/10 |
| Maintainability | 9/10 |
| Extensibility | 9/10 |
| User Experience | 8/10 |
| Documentation | 9/10 |
| Testing | 8/10 |
| Open Source Readiness | 9/10 |

Overall: 8.6/10.

This is strong enough for an honest `v0.1.0` public preview. It is not a 10/10 stable release until the plugin trust model, release binaries, workflow permissions, and external security review are complete.

## Resolved Launch Blockers

### Arbitrary Shell Execution

Previous severity: Critical

Status: Hardened

Why it mattered: A terminal AI tool that can execute shell commands can destroy user data if workflows, plugins, or copied commands abuse it.

Fix: `forge tool shell` is disabled by default and requires `[tools].shell_enabled = true`.

Remaining risk: Once enabled, shell execution is still intentionally powerful.

Effort: Implemented

### Unsafe File Writes

Previous severity: High

Status: Hardened

Why it mattered: `forge tool write` could write to arbitrary paths.

Fix: File writes now refuse to leave the current workspace by default. Users must opt into `[tools].allow_write_outside_workspace = true`.

Remaining risk: Workspace-local overwrites are still possible by design.

Effort: Implemented

### Plugin Path Traversal

Previous severity: High

Status: Hardened

Why it mattered: Plugin names could be used to build filesystem paths.

Fix: Plugin names now require a strict slug format and plugin paths are checked against the canonical plugin directory.

Remaining risk: Plugins are still executable/trusted content once installed.

Effort: Implemented

### Pattern And Workflow Path Traversal

Previous severity: Medium

Status: Hardened

Why it mattered: Pattern and workflow names are used as file names.

Fix: Pattern and workflow names now require letters, numbers, hyphen, or underscore.

Remaining risk: Installed YAML content still needs semantic validation.

Effort: Implemented

### Project Index Scalability

Previous severity: Medium

Status: Hardened

Why it mattered: Indexing could read large or binary files into memory.

Fix: Project indexing skips oversized files and binary-looking files. The limit is configurable through `[project].max_index_file_bytes`.

Remaining risk: Very large repositories still need cached incremental indexing.

Effort: Implemented

### Search Scalability

Previous severity: Medium

Status: Hardened

Why it mattered: Search could read large files as UTF-8 text.

Fix: Search skips files larger than `[tools].max_search_file_bytes`.

Remaining risk: Search is still simple substring search, not ripgrep-fast.

Effort: Implemented

### Provider Test Cost Visibility

Previous severity: Medium

Status: Hardened

Why it mattered: Provider tests can make paid API calls.

Fix: `forge provider test` now warns that configured remote providers receive a tiny live prompt that may use paid credits.

Remaining risk: Users can still intentionally run paid tests.

Effort: Implemented

### Unavailable Default Model

Previous severity: Medium

Status: Hardened

Why it mattered: A default model that does not exist makes first-run setup look broken.

Fix: Default changed from `gpt-5.5` to `gpt-4.1-mini`, and docs now frame model configuration explicitly.

Remaining risk: Provider model catalogs change over time.

Effort: Implemented

### Missing Threat Model

Previous severity: High

Status: Resolved for preview

Why it mattered: This CLI touches secrets, files, shell execution, memory, remote APIs, workflows, and plugins.

Fix: Added `docs/THREAT_MODEL.md`.

Remaining risk: Needs external security review before stable release.

Effort: Implemented

### Weak Contributor Onboarding

Previous severity: Medium

Status: Resolved for preview

Why it mattered: Open-source contributors need expectations, templates, checks, and a roadmap.

Fix: Added `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `ROADMAP.md`, `CHANGELOG.md`, issue templates, and PR template.

Remaining risk: Maintainers still need labels, triage cadence, and governance once community arrives.

Effort: Implemented

### Release Automation

Previous severity: High

Status: Partially resolved

Why it mattered: Most CLI users expect binaries, checksums, and CI.

Fix: Added CI and a tag-based release workflow that builds Linux, macOS, and Windows artifacts with SHA-256 checksums.

Remaining risk: The release workflow must be exercised with a real tag.

Effort: Implemented

### Testing Coverage

Previous severity: High

Status: Improved

Why it mattered: Provider parser tests alone did not prove CLI behavior.

Fix: Added CLI integration tests for provider listing, shell opt-in, plugin path traversal, and workspace write safety.

Remaining risk: Workflow, memory, and project indexing need deeper black-box tests.

Effort: Implemented

## Remaining Issues

### Plugin Trust Model

Severity: High

Why it matters: Plugins are still local executable content without signing, permission prompts, or lockfiles.

Concrete fix: Add plugin permissions, registry signatures, lockfile pinning, and install warnings.

Effort: Large

### Workflow Permissions

Severity: High

Why it matters: Workflows can become dangerous once they run tools and agents, especially when shared publicly.

Concrete fix: Add typed workflow steps, per-step permissions, and dry-run review.

Effort: Large

### Memory Privacy

Severity: Medium

Why it matters: Memory persists potentially sensitive notes and outputs in SQLite.

Concrete fix: Add retention policies, export, redaction, and optional encryption.

Effort: Medium

### Cost And Token Visibility

Severity: Medium

Why it matters: Agents and chains can make multiple paid provider calls.

Concrete fix: Add dry-run call estimates, token budgets, and provider-specific usage reporting where available.

Effort: Medium

### Incremental Project Index

Severity: Medium

Why it matters: Re-indexing large repositories is wasteful.

Concrete fix: Cache index entries by path, mtime, size, and hash.

Effort: Medium

## Release Plan

### Pre-Release Checklist

- Run `cargo fmt --check`.
- Run `cargo clippy -- -D warnings`.
- Run `cargo test`.
- Run smoke commands with a clean `FORGEFLOW_HOME`.
- Review `docs/THREAT_MODEL.md`.
- Verify release workflow with a prerelease tag.
- Confirm README install steps on Linux, macOS, and Windows.
- Create `v0.1.0` tag.
- Publish GitHub release artifacts.
- Announce as experimental public preview.

### GitHub Repository Structure

```text
.github/
  ISSUE_TEMPLATE/
  workflows/
CHANGELOG.md
CODE_OF_CONDUCT.md
CONTRIBUTING.md
LICENSE
README.md
ROADMAP.md
SECURITY.md
docs/
examples/
src/
tests/
Cargo.toml
Cargo.lock
```

### Recommended License

MIT. It is simple, permissive, familiar, and low-friction for a developer CLI.

### CI/CD

CI runs:

- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`

Release workflow builds:

- Linux x86_64
- macOS x86_64
- Windows x86_64
- SHA-256 checksums

### Versioning Strategy

Use SemVer.

- `0.1.x`: public preview
- `0.2.x`: workflow and plugin schema iteration
- `1.0.0`: stable CLI/config/plugin/workflow contracts

## Brutal Assessment

### NetworkChuck Audience

Why they install it: The demo is obvious: install, add key, run `forge agent debug-project`, get a useful terminal AI workflow.

Why they ignore it: If there is no one-line binary install, many viewers will not touch it.

What earns a star: A 60-second demo with Ollama and no cloud bill.

What causes uninstall: API key confusion or a scary shell warning without clear explanation.

### ThePrimeagen Audience

Why they install it: Rust, terminal-first, no Electron, no SaaS lock-in.

Why they ignore it: If agents only produce AI prose without touching real code/test loops, it gets dismissed as wrapperware.

What earns a star: Fast project indexing, visible commands, clear failures, no hidden magic.

What causes uninstall: Slow scans, weak tests, or vague "AI will fix it" output.

### Fireship Audience

Why they install it: The pitch is clean: Fabric plus workflows plus agents in your terminal.

Why they ignore it: Too many commands without a killer quickstart.

What earns a star: A compact README and one impressive copy-paste flow.

What causes uninstall: Dependency friction or provider setup pain.

### Random Rust Developer

Why they install it: The code is readable, modules are clear, and CI is conventional.

Why they ignore it: The plugin and workflow trust story still needs maturity.

What earns a star: Strong safety defaults and serious tests.

What causes uninstall: Panics, clippy warnings, unsafe path handling, or sloppy docs.

## Launch Verdict

FlowForge is now suitable for a serious `v0.1.0` public preview. Do not call it stable yet. The next jump toward a real 10/10 is not more providers; it is trusted plugins, typed workflows, release binaries, cost visibility, and external security review.
