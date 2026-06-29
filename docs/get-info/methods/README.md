# Getting Limits

## Purpose

This document describes options for retrieving usage/limits from AI providers and serves as an entry point into provider-specific documentation.

The product goal is to show the user current limits locally with minimal setup. Different providers expose data through different channels, so for each provider we document several possible approaches: primary, fallback, and research options.

## Terms

- Usage — current consumption for the period.
- Limit — available quota for the plan or included usage.
- Reset — when the period resets.
- Quota/rate limit — technical API or service constraints.
- Provider method — a specific channel for retrieving data for a provider.

## Data retrieval options

| Option | Status | Pros | Cons | Providers |
|---|---|---|---|---|
| Official API | Preferred when available | Stability, clear support, low risk | Often requires a key, enabling the API, or an Enterprise plan | Cursor Enterprise; potentially Codex/Claude/API providers |
| Local transcript/telemetry files | Candidate for MVP usage history | Fast, local, no network requests and no quota consumption | Usually gives usage history, but not always the official remaining limit/reset | Claude (`~/.config/claude/projects`, `~/.claude/projects`, Xcode ClaudeAgentConfig), Codex (`~/.codex`), Gemini (`~/.gemini/tmp`) |
| Statusline/hook stdin | Candidate for live limits | Can provide an official live signal (`rate_limits`) without TUI parsing | Works only inside a supported hook context; requires setup | Claude Code |
| Local derived DB/cache | Auxiliary layer | Speeds up the dashboard, incremental recalculation, and history after upstream cleanup | Not the primary source of truth; must stay in sync with source files | Claude (`~/.claude/usage.db`, tool-specific caches) |
| Provider CLI | Used in PoC | Works in a minimal user scenario using an already configured CLI | Slow, fragile TUI parsing, requests may consume resources | Codex, Claude; Cursor partially |
| Local token/client backend | Used in PoC for Cursor | Can work without a separate API key, using an existing login | Unofficial contract, subject to change, needs careful security review | Cursor |
| Frontend/dashboard API via cookie | Research-only | Often exposes the same data as the web UI | Cookie is a sensitive secret, high security and ToS risk | Potentially Codex, Claude, Cursor |
| Traffic observation | Research-only | Can help understand internal contracts | HTTPS, certificate pinning, high risk of fragility and misuse | Potentially all |

## Current status by provider

| Provider | Primary known option | Status | Documents |
|---|---|---|---|
| Codex | CLI `/status` | Implemented in PoC | [../providers/codex.md](../providers/codex.md) |
| Claude | CLI `/usage` + statusline hook stdin (`rate_limits`) | CLI implemented in PoC; statusline hook is a live-limit candidate | [../providers/claude.md](../providers/claude.md) |
| Cursor | `api2.cursor.sh` `GetCurrentPeriodUsage` via Cursor Agent token | Implemented in PoC; `cursor agent about/status` remains fallback | [../providers/cursor.md](../providers/cursor.md), [../../references/cursor-api2-cursor-sh.md](../../references/cursor-api2-cursor-sh.md) |

## Method selection principles

- Prefer the official, documented approach when it is available to the user without materially degrading UX.
- For a minimal scenario, prioritize an already installed and authorized local tool.
- Do not extract cookies, session tokens, or refresh tokens without explicit user consent and a separate threat model.
- Do not treat unofficial endpoints as a stable public contract.
- For each provider method, document data quality: which fields are available, how accurate they are, whether reset is included, and how often data can be refreshed.
- Implement fallback only if it improves the user scenario and does not disproportionately increase security/ToS risk.

## Related documents

- [from-provider-cli.md](from-provider-cli.md) — technical model for provider methods that retrieve data via the provider CLI/TUI.
- [from-local-files.md](from-local-files.md) — technical model for provider methods that retrieve data from local transcript/telemetry files.
- [from-statusline-hook.md](from-statusline-hook.md) — technical model for provider methods that retrieve live limits from statusline hook stdin payload.
- [../providers/codex.md](../providers/codex.md) — ways to retrieve Codex limits.
- [../providers/claude.md](../providers/claude.md) — ways to retrieve Claude limits.
- [../providers/cursor.md](../providers/cursor.md) — ways to retrieve Cursor limits.
