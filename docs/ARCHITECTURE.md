# ForgeFlow Architecture

## 1. Full Architecture

ForgeFlow is split into small terminal-first subsystems:

| Layer | Responsibility |
| --- | --- |
| CLI | Parse commands, print rich terminal output, stream task progress. |
| Config | Resolve `~/.forgeflow/config.toml`, provider/model defaults, feature toggles. |
| Providers | Stream OpenAI, Anthropic, Gemini, Ollama, and OpenRouter behind one `ProviderClient` trait. |
| Patterns | Load Fabric-style YAML patterns, render prompts, run single patterns or chains. |
| Workflows | Execute YAML step lists where each step maps to a pattern, tool, or agent. |
| Agents | Coordinate specialized agents such as Planner, Researcher, Coder, Tester, Reviewer, and Documenter. |
| Memory | Persist notes, outputs, project summaries, and user preferences in SQLite. |
| Tools | Provide controlled terminal tools: file IO, search, git, shell, HTTP, and project scan. |
| Plugins | Discover extension packages that add patterns, agents, providers, tools, workflow steps, and commands. |
| Project Awareness | Index repositories, inspect dependencies, summarize architecture, and provide context to patterns and agents. |

The guiding principle is that every feature must be useful from a terminal, pipe, script, or CI job.

## 2. Folder Structure

```text
.
├── Cargo.toml
├── README.md
├── docs/
│   └── ARCHITECTURE.md
├── examples/
│   ├── patterns/
│   │   ├── architecture.yaml
│   │   ├── debug.yaml
│   │   ├── explain.yaml
│   │   ├── flashcards.yaml
│   │   ├── quiz.yaml
│   │   ├── refactor.yaml
│   │   └── summarize.yaml
│   └── workflows/
│       ├── bug-fix-pipeline.yaml
│       └── docs-pipeline.yaml
└── src/
    ├── main.rs
    ├── agents.rs
    ├── cli.rs
    ├── config.rs
    ├── memory.rs
    ├── patterns.rs
    ├── plugins.rs
    ├── project.rs
    ├── providers.rs
    ├── terminal.rs
    ├── tools.rs
    └── workflows.rs
```

## 3. Rust Implementation Plan

Phase 1 ships a working CLI with `clap`, config loading, provider selection, builtin patterns, and pattern execution. Phase 2 adds real streaming provider clients, chaining, and SQLite memory. Phase 3 expands multi-agent context passing. Phase 4 adds repository indexing with file hashes and symbol hints. Phase 5 adds plugin SDK scaffolding and validation. Phase 6 introduces a local workflow marketplace as the registry path.

Core crates:

| Need | Crate |
| --- | --- |
| CLI parsing | `clap` |
| Async runtime | `tokio` |
| HTTP | `reqwest` |
| Terminal output | `console`, `indicatif`, later `ratatui` |
| Config | `toml`, `serde` |
| Workflow and patterns | `serde_yaml` |
| Memory | `rusqlite` |
| Project scan | `ignore` |

## 4. Database Schema

SQLite database: `~/.forgeflow/forgeflow.sqlite`.

```sql
CREATE TABLE memories (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  content TEXT NOT NULL,
  project_path TEXT,
  metadata_json TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL
);

CREATE INDEX idx_memories_kind ON memories(kind);
CREATE INDEX idx_memories_project_path ON memories(project_path);
CREATE VIRTUAL TABLE memories_fts USING fts5(content, kind, project_path, content='memories', content_rowid='rowid');
```

MVP uses a portable `LIKE` search, with FTS ready to add once migration management is introduced.

## 5. Pattern Engine

Patterns are YAML files:

```yaml
name: summarize
description: Condense input into the most useful points.
system_prompt: You summarize technical material for developers.
user_template: |
  Summarize the following:

  {{input}}
```

The engine resolves patterns in this order:

1. User patterns in `~/.forgeflow/patterns`.
2. Plugin-provided patterns.
3. Builtin fallback patterns.

Pattern chaining passes the previous pattern output into the next pattern as `{{input}}`, enabling commands like:

```bash
cat notes.md | forge chain summarize quiz flashcards
```

## 6. Workflow Engine

Workflows are YAML files in `~/.forgeflow/workflows`:

```yaml
name: bug-fix-pipeline
steps:
  - analyze
  - debug
  - fix
  - review
```

MVP treats each step name as a pattern. The next implementation should support typed steps:

```yaml
steps:
  - pattern: debug
  - tool: git.diff
  - agent: Coder
  - pattern: review
```

## 7. Agent Orchestration

Agents have a shared contract:

```rust
trait Agent {
    fn name(&self) -> &'static str;
    async fn run(&self, ctx: AgentContext) -> Result<AgentOutput>;
}
```

Built-in agents:

| Agent | Role |
| --- | --- |
| Planner | Break goals into steps. |
| Researcher | Gather project and external context. |
| Coder | Propose and apply code changes. |
| Tester | Run verification and interpret failures. |
| Reviewer | Check correctness, risks, and missing tests. |
| Documenter | Produce docs and release notes. |

Agent pipelines pass structured outputs forward, not just plain text.

## 8. Plugin Architecture

Plugins live under `~/.forgeflow/plugins/<name>` and include:

```toml
name = "example"
version = "0.1.0"

[capabilities]
patterns = true
agents = false
providers = false
tools = true
workflow_steps = true
commands = false
```

Directory layout:

```text
plugin/
├── plugin.toml
├── patterns/
├── workflows/
├── tools/
└── bin/
```

MVP discovers metadata. Later phases add signed installation, sandboxed execution, version pinning, and a registry.

## 9. Configuration System

Config path: `~/.forgeflow/config.toml`.

```toml
provider = "openai"
model = "gpt-4.1-mini"
temperature = 0.7
max_tokens = 2048
timeout_secs = 120
retries = 2
memory_enabled = true
plugins_enabled = true

[providers.openai]
api_key = "${OPENAI_API_KEY}"

[providers.ollama]
host = "http://localhost:11434"
model = "llama3.2"

[tools]
shell_enabled = false
allow_write_outside_workspace = false

[project]
max_index_file_bytes = 1048576
```

Provider secrets stay in environment variables by default.

## 10. Example Commands

```bash
forge provider set openai
forge provider list
forge provider info openai
forge provider test openai
forge run summarize < README.md
forge chain summarize quiz flashcards --text "Ownership in Rust is..."
forge pattern create release-notes
forge workflow create bug-fix-pipeline
forge workflow marketplace
forge workflow install docs-pipeline
forge workflow run bug-fix-pipeline
forge agent debug-project
forge analyze
forge architecture
forge remember "Use OpenRouter for experiments."
forge memory search OpenRouter
forge plugin list
forge plugin scaffold my-plugin
forge plugin validate ~/.forgeflow/plugins/my-plugin
forge tool search "TODO"
forge tool git status
forge tool http https://example.com
```

## 11. MVP Roadmap

| Phase | Scope | Exit Criteria |
| --- | --- | --- |
| 1 | CLI, config, provider abstraction, basic patterns | `forge run summarize` works with stdin and provider stubs. |
| 2 | Providers, chaining, memory | Streaming provider clients, `forge chain`, and SQLite-backed `remember/search` work. |
| 3 | Real agent orchestration | Built-in agent pipelines receive project index context and pass structured outputs. |
| 4 | Project indexing | Repository files, hashes, manifests, extensions, and symbol hints are indexed. |
| 5 | Plugin SDK | Plugin scaffolding and manifest validation work. |
| 6 | Workflow marketplace | Local marketplace listing, install, and publish validation work. |

## 12. Initial Codebase Skeleton

The current repository implements compile-ready modules for every subsystem. Remote providers are tested with mocked streaming HTTP responses so the CLI, patterns, workflows, and memory remain verifiable without paid API keys.
