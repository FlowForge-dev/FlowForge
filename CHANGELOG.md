# Changelog

All notable changes to FlowForge are documented here.

## 0.1.0 - Unreleased

Initial public preview.

### Added

- Terminal-first `forge` CLI.
- Streaming provider integrations for OpenAI, Anthropic, Gemini, Ollama, and OpenRouter.
- Fabric-style YAML patterns and pattern chaining.
- YAML workflows and local workflow marketplace commands.
- Project-aware agent orchestration.
- SQLite-backed memory.
- Plugin scaffold and manifest validation.
- Project indexing with hashes, manifests, extensions, line counts, and symbol hints.
- Launch docs, CI, security policy, contribution guide, and release readiness report.

### Hardened

- Shell tool is disabled by default and requires explicit config opt-in.
- File writes refuse to leave the current workspace by default.
- Plugin, pattern, and workflow names reject path traversal.
- Search and project indexing skip oversized files by default.
- Project indexing skips binary-looking files.
- CLI integration tests cover critical first-run and safety behavior.
