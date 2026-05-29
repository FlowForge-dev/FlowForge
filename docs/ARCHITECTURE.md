# Architecture

**Remember:** FlowForge is a small CLI made of simple parts.

## The Big Picture

```text
user
  -> forge command
  -> pattern / workflow / agent
  -> provider
  -> streamed answer
```

## The Parts

| Part | Job |
| --- | --- |
| CLI | Reads commands like `forge chat`. |
| Config | Loads `~/.forgeflow/config.toml`. |
| Providers | Talks to OpenAI, Anthropic, Gemini, Ollama, OpenRouter. |
| Patterns | Runs saved prompts. |
| Workflows | Runs steps in order. |
| Agents | Runs role-based helpers. |
| Project Index | Scans the current repo. |
| Memory | Stores notes in SQLite. |
| Plugins | Adds community extensions. |
| Tools | Reads files, searches, uses git, HTTP, and optional shell. |

## Important Files

```text
src/main.rs        starts the app
src/cli.rs         command list
src/config.rs      config file
src/providers.rs   AI providers
src/patterns.rs    saved prompts
src/workflows.rs   YAML workflows
src/agents.rs      agent pipelines
src/project.rs     project scanner
src/memory.rs      SQLite memory
src/plugins.rs     plugin SDK
src/tools.rs       local tools
```

## Config

```toml
provider = "openai"
model = "gpt-4.1-mini"

[providers.openai]
api_key = "${OPENAI_API_KEY}"

[tools]
shell_enabled = false
allow_write_outside_workspace = false
```

## Pattern

**Remember:** pattern = saved prompt.

```yaml
name: summarize
description: Summarize text.
system_prompt: You summarize for developers.
user_template: |
  Summarize this:

  {{input}}
```

FlowForge ships a practical pattern pack:

```text
distill
key-points
insight-map
action-items
decision-brief
study-notes
concept-map
claim-check
counterpoints
quote-bank
question-set
checklist
meeting-brief
research-brief
newbie-explain
risks
next-steps
content-ideas
```

## Workflow

**Remember:** workflow = list of steps.

```yaml
name: docs-pipeline
steps:
  - summarize
  - architecture
  - explain
```

Demo workflows:

```text
knowledge-pipeline
study-pipeline
decision-pipeline
content-pipeline
```

## Agent

**Remember:** agent = role with a job.

Agents pass their output to the next agent:

```text
Planner -> Coder -> Tester -> Reviewer
```

## Storage

Created automatically on first run:

```text
Windows:
C:\Users\<you>\.forgeflow\config.toml       config
C:\Users\<you>\.forgeflow\forgeflow.sqlite  memory
C:\Users\<you>\.forgeflow\patterns\         user patterns
C:\Users\<you>\.forgeflow\workflows\        user workflows
C:\Users\<you>\.forgeflow\plugins\          plugins

macOS / Linux:
~/.forgeflow/config.toml       config
~/.forgeflow/forgeflow.sqlite  memory
~/.forgeflow/patterns/         user patterns
~/.forgeflow/workflows/        user workflows
~/.forgeflow/plugins/          plugins
```

## Safety

Default stance:

- do not run shell unless enabled
- do not write outside the workspace unless enabled
- do not trust plugins blindly
- do not put raw API keys in files
