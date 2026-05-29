# FlowForge

FlowForge is the GitHub home for **ForgeFlow**, a terminal-first AI automation platform for developers.

ForgeFlow is not an AI model. It is a CLI harness that lets you bring your own provider keys and run reusable patterns, chains, workflows, project-aware agents, memory, plugins, and terminal tools.

No web UI. No Electron. No desktop app. Just the terminal.

## What You Can Do

```bash
forge run summarize < README.md
forge chain summarize explain --text "Explain this release note."
forge chat
forge agent build-app --goal "Create a small Rust CLI"
forge agent debug-project
forge explain-project
forge workflow run docs-pipeline
```

## Install

### 1. Install Rust

Install Rust from [rustup.rs](https://rustup.rs/).

Check it:

```bash
rustc --version
cargo --version
```

### 2. Clone FlowForge

```bash
git clone https://github.com/AndroRAT-user/FlowForge.git
cd FlowForge
```

### 3. Build the CLI

```bash
cargo build
```

### 4. Run it locally

```bash
cargo run -- provider list
cargo run -- pattern list
cargo run -- run summarize --text "ForgeFlow is a terminal-first AI automation harness."
```

### 5. Install the `forge` command

```bash
cargo install --path .
forge provider list
```

## Configure A Provider

ForgeFlow reads config from:

```text
~/.forgeflow/config.toml
```

Minimal OpenAI example:

```toml
provider = "openai"
model = "gpt-5.5"
temperature = 0.7
max_tokens = 2048
timeout_secs = 120
retries = 2

[providers.openai]
api_key = "${OPENAI_API_KEY}"
```

Then set your environment variable:

```bash
export OPENAI_API_KEY="sk-..."
forge provider test openai
```

PowerShell:

```powershell
$env:OPENAI_API_KEY = "sk-..."
forge provider test openai
```

## Supported Providers

```bash
forge provider list
forge provider info openai
forge provider test openai
```

Supported providers:

- OpenAI
- Anthropic
- Gemini
- Ollama
- OpenRouter

Ollama example:

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

## Patterns

Patterns are reusable YAML prompts:

```bash
forge run summarize < notes.md
forge run explain --text "Rust ownership"
forge chain summarize quiz flashcards < tutorial.md
```

Manage patterns:

```bash
forge pattern list
forge pattern create release-notes
forge pattern edit release-notes
forge pattern install ./my-pattern.yaml
forge pattern remove release-notes
```

## Workflows

Workflows are YAML pipelines:

```bash
forge workflow list
forge workflow marketplace
forge workflow install docs-pipeline
forge workflow run docs-pipeline < README.md
```

Example:

```yaml
name: bug-fix-pipeline
description: Analyze, debug, fix, and review a bug report.
steps:
  - analyze
  - debug
  - fix
  - review
```

## Agents

```bash
forge agent build-app --goal "Build a terminal Markdown summarizer"
forge agent debug-project
forge agent review-code
```

Agents receive project index context and pass outputs between roles:

- Planner
- Researcher
- Coder
- Tester
- Reviewer
- Documenter

## Project Awareness

Run inside a repository:

```bash
forge analyze
forge explain-project
forge architecture
forge find-dead-code
forge generate-docs
```

The project index includes:

- files
- extensions
- manifests
- line counts
- SHA-256 hashes
- symbol hints

## Memory

```bash
forge remember "Prefer short terminal output."
forge recall
forge memory search terminal
forge memory clear
```

Memory is stored in SQLite under `~/.forgeflow`.

## Plugins

```bash
forge plugin scaffold my-plugin
forge plugin validate ~/.forgeflow/plugins/my-plugin
forge plugin list
```

Plugins can provide patterns, workflows, tools, agents, providers, workflow steps, and commands.

## Development

Run the full local check:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

Useful smoke tests:

```bash
forge provider list
forge workflow marketplace
forge plugin scaffold smoke-plugin
forge plugin validate ~/.forgeflow/plugins/smoke-plugin
```

## Repository Map

```text
src/
  agents.rs      agent orchestration
  cli.rs         clap command definitions
  config.rs      TOML config and provider settings
  memory.rs      SQLite memory
  patterns.rs    Fabric-style pattern engine
  plugins.rs     plugin SDK scaffolding and validation
  project.rs     project indexing
  providers.rs   streaming provider integrations
  tools.rs       terminal tool framework
  workflows.rs   YAML workflow engine
docs/
  ARCHITECTURE.md
  RELEASE_READINESS_REPORT.md
examples/
  patterns/
  workflows/
```

## Security

Do not put raw API keys in public config files. Prefer environment references such as:

```toml
api_key = "${OPENAI_API_KEY}"
```

Report vulnerabilities privately using the process in [SECURITY.md](SECURITY.md).

## License

FlowForge is released under the MIT License. See [LICENSE](LICENSE).
