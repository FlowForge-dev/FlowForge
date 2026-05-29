# Security Policy

## Supported Versions

Until the first stable release, only the latest `main` branch is supported.

## Reporting A Vulnerability

Please do not open a public issue for security vulnerabilities.

Email the maintainers or use GitHub private vulnerability reporting when enabled. Include:

- affected command or subsystem
- steps to reproduce
- impact
- whether secrets, files, shell execution, or network access are involved
- suggested fix if known

You should receive an initial response within 7 days.

## Security Expectations

- Do not commit API keys or provider tokens.
- Prefer `api_key = "${ENV_VAR}"` in config.
- Treat plugins as executable code.
- Review workflow files before running them from untrusted sources.
- Be careful with `forge tool shell` and file write commands.
