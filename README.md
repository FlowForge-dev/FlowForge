# FlowForge

**Remember:** FlowForge gives you one command, `forge`, for AI work in your terminal.

No web app. No desktop app. No login screen. Bring your own AI key.

## 60-Second Install

Use this:

```bash
curl -fsSL https://raw.githubusercontent.com/FlowForge-dev/FlowForge/main/install.sh | sh
forge provider list
```

Windows PowerShell:

```powershell
irm https://raw.githubusercontent.com/FlowForge-dev/FlowForge/main/install.ps1 | iex
forge provider list
```

If no release binary exists yet, the installer builds from source. If it asks for Rust, install Rust from [rustup.rs](https://rustup.rs/) and run the installer again.

After install, stay anywhere. Do **not** `cd flowforge`. Just run:

```powershell
forge provider list
notepad $HOME\.forgeflow\config.toml
```

Your first `forge` command creates this automatically:

```text
Windows:
%USERPROFILE%\.forgeflow\config.toml
%USERPROFILE%\.forgeflow\patterns\
%USERPROFILE%\.forgeflow\workflows\
%USERPROFILE%\.forgeflow\plugins\

macOS / Linux:
~/.forgeflow/config.toml
~/.forgeflow/patterns/
~/.forgeflow/workflows/
~/.forgeflow/plugins/
```

## 60-Second Setup

Open the config file that already exists:

```text
Windows:
%USERPROFILE%\.forgeflow\config.toml

macOS / Linux:
~/.forgeflow/config.toml
```

Windows:

```powershell
notepad $HOME\.forgeflow\config.toml
```

It already has safe defaults. For OpenAI, make sure it contains:

```toml
provider = "openai"
model = "gpt-4.1-mini"

[providers.openai]
api_key = "${OPENAI_API_KEY}"
```

Set your key:

```bash
export OPENAI_API_KEY="sk-..."
forge provider test openai
```

PowerShell:

```powershell
$env:OPENAI_API_KEY = "sk-..."
forge provider test openai
```

## First 5 Commands

Memorize these:

```bash
forge chat
forge run summarize < README.md
forge run explain --text "Rust ownership"
forge explain-project
forge agent debug-project
```

## The Cheat Sheet

| Want to... | Run this |
| --- | --- |
| Chat | `forge chat` |
| Summarize text | `forge run summarize < file.md` |
| Explain something | `forge run explain --text "topic"` |
| Chain prompts | `forge chain summarize explain < file.md` |
| Inspect a project | `forge explain-project` |
| Debug a project | `forge agent debug-project` |
| Run a workflow | `forge workflow run docs-pipeline < README.md` |
| Save a note | `forge remember "Use short output"` |
| Search memory | `forge memory search short` |
| List providers | `forge provider list` |

## What Is A Pattern?

**Remember:** a pattern is a saved prompt.

```bash
forge pattern list
forge run summarize < notes.md
forge run debug < error.log
```

## What Is A Workflow?

**Remember:** a workflow is a list of patterns.

```bash
forge workflow marketplace
forge workflow install docs-pipeline
forge workflow run docs-pipeline < README.md
```

## What Is An Agent?

**Remember:** an agent is a role with a job.

Examples:

```bash
forge agent build-app --goal "Build a tiny CLI"
forge agent debug-project
forge agent review-code
```

Built-in agents:

- Planner
- Researcher
- Coder
- Tester
- Reviewer
- Documenter

## Use Ollama Instead

```toml
provider = "ollama"
model = "llama3.2"

[providers.ollama]
host = "http://localhost:11434"
```

Then:

```bash
ollama serve
ollama pull llama3.2
forge provider test ollama
```

## Safety Defaults

**Remember:** dangerous tools are off by default.

```toml
[tools]
shell_enabled = false
allow_write_outside_workspace = false
```

Enable shell only if you know what you are doing:

```toml
[tools]
shell_enabled = true
```

## Learn More

- [Troubleshooting](docs/TROUBLESHOOTING.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Threat Model](docs/THREAT_MODEL.md)
- [Contributing](CONTRIBUTING.md)
- [Roadmap](ROADMAP.md)

## License

MIT. See [LICENSE](LICENSE).
