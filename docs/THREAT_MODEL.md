# Threat Model

**Remember:** FlowForge is powerful, so it must be careful.

## What We Protect

- API keys
- source code
- local files
- shell environment
- memory database
- workflows
- plugins

## Risky Areas

| Area | Why it matters |
| --- | --- |
| Providers | Your prompts leave your machine. |
| Shell | Commands can change or delete files. |
| File tools | Reads and writes touch your project. |
| Plugins | Plugins may run code. |
| Workflows | Workflows can chain many actions. |
| Memory | Notes may contain private data. |

## Current Safety Defaults

- Shell is off by default.
- File writes stay inside the workspace by default.
- Unsafe names are blocked.
- Large and binary files are skipped during indexing.
- API keys can use environment variables.

## Still Not Perfect

- Plugins are not sandboxed yet.
- Memory is not encrypted yet.
- Workflow permissions are not complete yet.
- Provider calls can cost money.

## Maintainer Rule

If a change touches shell, files, plugins, memory, or secrets, treat it as security-sensitive.
