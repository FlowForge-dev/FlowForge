# Threat Model

FlowForge is powerful because it connects AI providers, local files, shell commands, workflows, memory, and plugins. That also makes it security-sensitive.

## Assets

- Provider API keys.
- Source code and local project files.
- Shell environment variables.
- Persistent memory database.
- Workflow and plugin files.
- Model outputs that may contain private context.

## Trust Boundaries

| Boundary | Risk |
| --- | --- |
| User config | May contain secrets or dangerous tool settings. |
| Provider APIs | Remote services receive prompt context. |
| Plugins | Plugin code and binaries may be untrusted. |
| Workflows | Workflow steps can trigger multiple model calls. |
| Shell tool | Can execute arbitrary local commands when enabled. |
| File tools | Can read or write local project files. |
| Memory | Persists potentially sensitive notes and outputs. |

## Attacker Models

- A malicious plugin author.
- A malicious workflow shared online.
- A prompt injection inside repository content or logs.
- A compromised provider key.
- Accidental user misuse through copied shell snippets.

## Current Mitigations

- Shell execution is disabled by default.
- File writes are limited to the current workspace by default.
- Plugin, pattern, and workflow names reject path traversal.
- Provider keys can be referenced through environment variables.
- Project indexing skips oversized and binary-looking files.
- CI runs format, clippy, and tests.

## Known Gaps

- Plugins are not sandboxed.
- Workflow steps are not permissioned.
- Memory is not encrypted.
- Provider calls do not estimate cost.
- There is no signed marketplace.

## Rules For Contributors

- Treat every workflow and plugin as untrusted input.
- Do not add new shell or file capabilities without a safety review.
- Prefer opt-in behavior for dangerous actions.
- Add tests for path handling and config behavior.
- Never log raw provider keys.
