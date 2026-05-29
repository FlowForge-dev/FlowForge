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
forge config open
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

```bash
forge config open
```

If your editor does not open, print the exact file path:

```bash
forge config path
```

The config already has safe defaults. For OpenAI, make sure it contains:

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

OpenRouter:

```powershell
forge provider configure openrouter --api-key "sk-or-..." --model "minimax/minimax-m2.5:free"
forge provider test openrouter
```

## First 5 Commands

Memorize these:

```bash
forge chat
forge run summarize < README.md
forge run distill < notes.md
forge run explain --text "Rust ownership"
forge explain-project
forge agent debug-project
```

## The Cheat Sheet

| Want to... | Run this |
| --- | --- |
| Chat | `forge chat` |
| Summarize text | `forge run summarize < file.md` |
| Extract useful ideas | `forge run distill < file.md` |
| Get action items | `forge run action-items < notes.md` |
| Make study notes | `forge run study-notes < article.md` |
| Build a decision memo | `forge run decision-brief < notes.md` |
| Explain something | `forge run explain --text "topic"` |
| Chain prompts | `forge chain summarize explain < file.md` |
| Inspect a project | `forge explain-project` |
| Debug a project | `forge agent debug-project` |
| Run knowledge workflow | `forge workflow run knowledge-pipeline < notes.md` |
| Run a workflow | `forge workflow run docs-pipeline < README.md` |
| Save a note | `forge remember "Use short output"` |
| Search memory | `forge memory search short` |
| List providers | `forge provider list` |
| Open config | `forge config open` |

## What Is A Pattern?

**Remember:** a pattern is a saved prompt.

```bash
forge pattern list
forge run summarize < notes.md
forge run distill < notes.md
forge run action-items < meeting.txt
forge run debug < error.log
```

## Pattern Commands

**Remember:** every pattern runs the same way: `forge run NAME < file`.

| Pattern | Use it for | Example |
| --- | --- | --- |
| `action-items` | Turn notes into tasks | `forge run action-items < meeting.txt` |
| `analyze` | Analyze problem context | `forge run analyze < context.md` |
| `architecture` | Explain system design | `forge run architecture < design.md` |
| `checklist` | Make a practical checklist | `forge run checklist < guide.md` |
| `claim-check` | Separate claims from evidence | `forge run claim-check < article.md` |
| `concept-map` | Map concepts and relationships | `forge run concept-map < notes.md` |
| `content-ideas` | Create honest content angles | `forge run content-ideas < transcript.txt` |
| `counterpoints` | Stress-test an idea | `forge run counterpoints < proposal.md` |
| `debug` | Find likely causes in errors | `forge run debug < error.log` |
| `decision-brief` | Make a decision memo | `forge run decision-brief < options.md` |
| `distill` | Extract useful knowledge | `forge run distill < notes.md` |
| `explain` | Explain a concept or code | `forge run explain --text "Rust ownership"` |
| `fix` | Propose a fix plan | `forge run fix < bug-report.md` |
| `flashcards` | Make study flashcards | `forge run flashcards < lesson.md` |
| `insight-map` | Map ideas and tradeoffs | `forge run insight-map < research.md` |
| `key-points` | Pull essentials only | `forge run key-points < article.md` |
| `meeting-brief` | Summarize a meeting | `forge run meeting-brief < transcript.txt` |
| `newbie-explain` | Explain for a beginner | `forge run newbie-explain --text "Docker containers"` |
| `next-steps` | Pick the next moves | `forge run next-steps < plan.md` |
| `question-set` | Create good questions | `forge run question-set < notes.md` |
| `quiz` | Make a short quiz | `forge run quiz < lesson.md` |
| `quote-bank` | Pull useful short quotes | `forge run quote-bank < source.txt` |
| `refactor` | Suggest safe refactors | `forge run refactor < code.rs` |
| `research-brief` | Organize research notes | `forge run research-brief < research.md` |
| `review` | Review risks and gaps | `forge run review < solution.md` |
| `risks` | Find risks and mitigations | `forge run risks < proposal.md` |
| `study-notes` | Make beginner notes | `forge run study-notes < article.md` |
| `summarize` | Summarize text | `forge run summarize < README.md` |

## What Is A Workflow?

**Remember:** a workflow is a list of patterns.

## Workflow Commands

**Remember:** every workflow runs the same way: `forge workflow run NAME < file`.

| Workflow | Steps | Example |
| --- | --- | --- |
| `bug-fix-pipeline` | `analyze -> debug -> fix -> review` | `forge workflow run bug-fix-pipeline < bug-report.md` |
| `content-pipeline` | `distill -> quote-bank -> content-ideas` | `forge workflow run content-pipeline < transcript.txt` |
| `decision-pipeline` | `key-points -> decision-brief -> risks -> counterpoints` | `forge workflow run decision-pipeline < notes.md` |
| `docs-pipeline` | `summarize -> architecture -> explain` | `forge workflow run docs-pipeline < README.md` |
| `knowledge-pipeline` | `distill -> insight-map -> action-items -> next-steps` | `forge workflow run knowledge-pipeline < notes.md` |
| `study-pipeline` | `newbie-explain -> study-notes -> question-set -> flashcards` | `forge workflow run study-pipeline < article.md` |

More workflow commands:

```bash
forge workflow list
forge workflow marketplace
forge workflow install docs-pipeline
forge workflow create my-pipeline
forge workflow edit my-pipeline
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
