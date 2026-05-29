# Troubleshooting

## `forge provider test openai` says the provider is not configured

Set the provider key as an environment variable and reference it in `~/.forgeflow/config.toml`.

```toml
[providers.openai]
api_key = "${OPENAI_API_KEY}"
```

PowerShell:

```powershell
$env:OPENAI_API_KEY = "sk-..."
forge provider test openai
```

## Ollama Is Unreachable

Start Ollama and make sure the configured host is correct.

```bash
ollama serve
ollama pull llama3.2
forge provider test ollama
```

Config:

```toml
provider = "ollama"
model = "llama3.2"

[providers.ollama]
host = "http://localhost:11434"
```

## Shell Tool Is Disabled

This is intentional. Enable it only if you understand the risk.

```toml
[tools]
shell_enabled = true
```

Then:

```bash
forge tool shell echo hello
```

## File Write Is Refused Outside The Workspace

By default, `forge tool write` will not write outside the current working directory.

To opt in:

```toml
[tools]
allow_write_outside_workspace = true
```

## Provider Calls Are Slow Or Fail Mid-Stream

Tune timeout and retry settings:

```toml
timeout_secs = 180
retries = 1
```

Retries can duplicate paid model calls, so keep this conservative.

## Project Scan Is Missing Large Files

FlowForge skips files larger than 1 MiB by default when indexing/searching.

```toml
[project]
max_index_file_bytes = 2097152

[tools]
max_search_file_bytes = 2097152
```

## Invalid Plugin, Pattern, Or Workflow Name

Names must use only letters, numbers, hyphen, or underscore.

Good:

```text
release-notes
docs_pipeline
```

Bad:

```text
../plugin
my/plugin
```
