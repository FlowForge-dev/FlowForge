# Security

**Remember:** FlowForge can touch files, API keys, AI providers, plugins, and shell commands.

## Report A Security Bug

Do not open a public issue.

Use GitHub private vulnerability reporting when available:

```text
https://github.com/FlowForge-dev/FlowForge/security/advisories/new
```

Include:

- what broke
- how to reproduce it
- what data is at risk
- whether files, shell, plugins, memory, or API keys are involved

## Safe Defaults

- Shell is off by default.
- File writes stay inside the workspace by default.
- API keys should come from environment variables.
- Plugins should be treated like executable code.

## Safer Config

Good:

```toml
api_key = "${OPENAI_API_KEY}"
```

Risky:

```toml
api_key = "sk-real-secret-here"
```

## Before Running Shared Workflows

Read them first.

If a workflow or plugin asks for shell access, slow down.
