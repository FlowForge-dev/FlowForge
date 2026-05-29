# Troubleshooting

**Remember:** most problems are setup problems.

## `cargo` Is Missing

The installer tried to build from source because a release binary was not available yet.

Install Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Windows: use [rustup.rs](https://rustup.rs/).

Then install FlowForge:

```bash
curl -fsSL https://raw.githubusercontent.com/FlowForge-dev/FlowForge/main/install.sh | sh
forge provider list
```

Windows:

```powershell
irm https://raw.githubusercontent.com/FlowForge-dev/FlowForge/main/install.ps1 | iex
forge provider list
```

## I Cannot `cd flowforge`

That is okay.

After install, you do not need the repo folder.

Run this from anywhere:

```powershell
forge provider list
forge config open
```

The installed program lives here:

```text
%USERPROFILE%\.cargo\bin\forge.exe
```

The config lives here:

```bash
forge config path
```

## Provider Is Not Configured

First make sure the starter files exist by running any normal command:

```bash
forge provider list
```

Open config:

```bash
forge config open
```

Set your key:

```bash
export OPENAI_API_KEY="sk-..."
```

PowerShell:

```powershell
$env:OPENAI_API_KEY = "sk-..."
```

Config:

```toml
[providers.openai]
api_key = "${OPENAI_API_KEY}"
```

Test:

```bash
forge provider test openai
```

## OpenRouter Is Skipped

Configure it with one command:

```powershell
forge provider configure openrouter --api-key "sk-or-..." --model "openrouter/free"
forge provider test openrouter
```

If you prefer environment variables:

```powershell
setx OPENROUTER_API_KEY "sk-or-..."
forge provider configure openrouter --api-key-env OPENROUTER_API_KEY --model "openrouter/free"
```

Close PowerShell, open it again, then run:

```powershell
forge provider test openrouter
```

## Ollama Is Unreachable

Start it:

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

Good. That is safer.

Enable it only if needed:

```toml
[tools]
shell_enabled = true
```

## File Write Was Blocked

FlowForge will not write outside your current folder by default.

To allow it:

```toml
[tools]
allow_write_outside_workspace = true
```

## Scan Missed A Large File

Large files are skipped by default.

Raise the limit:

```toml
[project]
max_index_file_bytes = 2097152
```

## Invalid Name

Names can use:

```text
letters numbers - _
```

Good:

```text
release-notes
docs_pipeline
```

Bad:

```text
../secret
my/plugin
```
