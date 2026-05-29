# Changelog

**Remember:** this file tells you what changed.

## 0.1.0 - Unreleased

First public preview.

### Added

- `forge` terminal CLI.
- OpenAI, Anthropic, Gemini, Ollama, and OpenRouter providers.
- Streaming output.
- Patterns.
- Pattern chains.
- Workflows.
- Agents.
- Project indexing.
- SQLite memory.
- Plugin scaffolding.
- Local workflow marketplace.
- CI, release workflow, and docs.
- Beginner install scripts.
- First run creates config and standard folders automatically.
- Beginner config commands: `forge config open`, `forge config path`, `forge config show`, and `forge config reset`.
- Beginner provider setup with `forge provider configure`.
- OpenRouter setup now defaults to the `openrouter/free` router and explains model-not-found errors.
- Knowledge extraction pattern pack.
- Demo workflows for knowledge, study, decisions, and content.

### Hardened

- Shell tool is off by default.
- File writes stay inside the workspace by default.
- Plugin, pattern, and workflow names reject unsafe paths.
- Large and binary files are skipped during indexing.
- CLI safety tests were added.
