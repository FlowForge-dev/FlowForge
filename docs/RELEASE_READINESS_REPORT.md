# Release Readiness

**Remember:** FlowForge is ready for a public preview, not a stable 1.0.

## Scores

| Area | Score |
| --- | ---: |
| Architecture | 9/10 |
| Security | 8/10 |
| Maintainability | 9/10 |
| Extensibility | 9/10 |
| User Experience | 8/10 |
| Documentation | 9/10 |
| Testing | 8/10 |
| Open Source Readiness | 9/10 |

Overall: **8.6/10**

## What Got Fixed

| Problem | Fix |
| --- | --- |
| Shell was too dangerous | Shell is off by default. |
| File writes were too broad | Writes stay inside the workspace by default. |
| Plugin paths were risky | Plugin names are validated. |
| Pattern/workflow paths were risky | Names are validated. |
| Big files could slow scans | Big files are skipped. |
| Binary files could pollute scans | Binary-looking files are skipped. |
| Provider tests could cost money | Tests now warn users. |
| Default model was confusing | Default is now `gpt-4.1-mini`. |
| No threat model | Added threat model. |
| Weak onboarding | Added roadmap, changelog, templates, docs. |
| Weak tests | Added CLI safety tests. |

## What Still Blocks 10/10

| Risk | Needed |
| --- | --- |
| Plugins are not sandboxed | Plugin permissions and signing. |
| Workflows need more safety | Typed steps and dry runs. |
| Memory is plain SQLite | Export, retention, optional encryption. |
| AI calls can cost money | Cost estimates and token budgets. |
| Indexing is not cached | Incremental project index. |
| Release workflow is new | Test it with a real tag. |

## Release Checklist

Before `v0.1.0`:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

Then:

- test install on a clean machine
- test OpenAI setup
- test Ollama setup
- create tag `v0.1.0`
- confirm release artifacts build
- announce as public preview

## Honest Verdict

Ship it as:

```text
FlowForge v0.1.0 Public Preview
```

Do not call it stable yet.

The next real step is not more providers. It is safer workflows, safer plugins, easier install, and clearer costs.
