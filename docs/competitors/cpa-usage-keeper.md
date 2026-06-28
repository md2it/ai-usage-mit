# CPA Usage Keeper

## META

- Date: 2026-06-28
- URL: https://github.com/Willxup/cpa-usage-keeper
- Relevance: Medium
- Pricing: Free (MIT, self-hosted only)
- License: MIT
- Authors: Willxup
- GitHub: 818 stars, 94 forks (as of 2026-06-28); latest release v1.12.1 (2026-06-26)
- Languages: Go 63%, TypeScript 31%, SCSS 6%

## OVERVIEW

CPA Usage Keeper is a **standalone observability sidecar for CLIProxyAPI (CPA)**. It is not a standalone product — it requires a running CLIProxyAPI instance (`CPA_BASE_URL` and `CPA_MANAGEMENT_KEY` are mandatory config). The service consumes usage events from CPA's Redis queue into a local SQLite database, periodically syncs CPA metadata (auth files, API keys, provider info), and serves a built-in React web dashboard for monitoring API consumption: request volume, token usage, cost, cache utilization, request health, model/provider breakdowns, latency diagnostics, and credential health snapshots. Actively developed — 55 releases, 6+ releases in the last two weeks of June 2026.

## PRODUCTS

| Repository | Description |
|---|---|
| [cpa-usage-keeper](https://github.com/Willxup/cpa-usage-keeper) | Single repo. Go backend + React/TypeScript frontend. |

## FEATURES

- SQLite persistence of CPA usage events consumed from Redis queue
- Web dashboard: request volume, token usage, cost, cache utilization, success rate, request performance
- Filterable request event log (by time range, model, API key, source, outcome) with streaming CSV/JSON export
- Trend analysis, cost breakdowns, provider composition visualization, hourly activity heatmaps
- Per-API-Key usage queries
- Credentials management: Auth Files and AI Provider credentials with quota inspection and refresh
- Model pricing maintenance with manual editing and models.dev sync
- Latency diagnostics: TTFT / total latency with p95 lines, density rendering
- Credential health snapshots (5-hour snapshots and graphs)
- Codex quota cache updates from CPA response headers
- Optional password-protected access (session tokens hashed SHA-256, persisted in SQLite)
- Automated SQLite backups and configurable data retention window
- macOS (Homebrew), Linux (binary + systemd), Docker / Docker Compose support

## PRICING / BUSINESS MODEL

Free, MIT-licensed, self-hosted only. No pricing tiers, no subscription, no commercial offering. No usage quotas or data retention limits stated in docs.

## COMPARISON WITH AI-USAGE-MIT

| Dimension | cpa-usage-keeper | ai-usage-mit |
|---|---|---|
| Standalone operation | No — requires CLIProxyAPI | Yes |
| Data source | CLIProxyAPI Redis queue only | Multiple sources (Claude Code, etc.) |
| Storage | SQLite (local) | TBD |
| Dashboard | Built-in React web UI | TBD |
| Pricing visibility | Cost tracking vs. models.dev | TBD |
| Credential health | Yes (5-hour snapshots) | TBD |
| CSV/JSON export | Yes (v1.12.1) | TBD |
| License | MIT | TBD |
| Platforms | macOS, Linux, Docker | TBD |
| Windows | Not mentioned | TBD |

**Threat level: Medium.** Directly addresses the usage observability gap for CLIProxyAPI users. Highly active development and solid feature set. However, it is permanently coupled to CLIProxyAPI — users not running CPA cannot use it at all. If ai-usage-mit targets broader data sources or standalone operation, overlap is limited to the CLIProxyAPI user segment.

## OPEN QUESTIONS

- Does cpa-usage-keeper plan to support data sources other than CLIProxyAPI Redis queue?
- Is there a hosted/SaaS version planned?
- What is the retention policy for SQLite data — is there a maximum window documented anywhere?

## SOURCES

- [GitHub repo](https://github.com/Willxup/cpa-usage-keeper)
- [README](https://raw.githubusercontent.com/Willxup/cpa-usage-keeper/main/README.md)
- [Releases / changelog](https://github.com/Willxup/cpa-usage-keeper/releases)
