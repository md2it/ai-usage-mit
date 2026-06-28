# CPA Usage Keeper

## CONCLUSION

### After documentation

- CPA Usage Keeper is a standalone observability sidecar for CLIProxyAPI (CPA).
- It requires a running CLIProxyAPI instance; `CPA_BASE_URL` and `CPA_MANAGEMENT_KEY` are mandatory.
- It provides a React web dashboard over CPA usage events stored in SQLite.
- It is actively developed and has a broad usage analytics feature set for CPA operators.

### After hands-on

Not checked yet.

### Comparison to ai-usage-mit

- Standalone operation: CPA Usage Keeper requires CLIProxyAPI; ai-usage-mit is expected to operate more broadly.
- Data source: CPA Usage Keeper reads from the CLIProxyAPI Redis queue only.
- Storage: CPA Usage Keeper uses local SQLite.
- Dashboard: CPA Usage Keeper has a built-in React web UI.
- Pricing visibility: CPA Usage Keeper tracks cost and can use models.dev pricing.
- Credential health: CPA Usage Keeper includes credential health snapshots.
- Export: CSV/JSON export is documented.
- Threat level: Medium. It directly addresses usage observability for CLIProxyAPI users and is actively developed, but its dependency on CLIProxyAPI limits overlap if ai-usage-mit targets broader standalone data sources.

### What we can learn

- A local SQLite event ledger is a practical model for searchable usage history.
- Filterable request logs and CSV/JSON export are useful for power users.
- Credential health and quota snapshots are relevant adjacent workflows for users managing multiple AI accounts.

## META

- Date: 2026-06-28
- URL: https://github.com/Willxup/cpa-usage-keeper
- Relevance: Medium
- Business model: Free, MIT-licensed, self-hosted only. No pricing tiers, subscription, commercial offering, usage quotas, or data retention limits are stated in docs.
- License: MIT
- Authors: Willxup
- GitHub: 818 stars, 94 forks (as of 2026-06-28); latest release v1.12.1 (2026-06-26)
- Languages: Go 63%, TypeScript 31%, SCSS 6%
- Sources:
  - [GitHub repo](https://github.com/Willxup/cpa-usage-keeper)
  - [README](https://raw.githubusercontent.com/Willxup/cpa-usage-keeper/main/README.md)
  - [Releases / changelog](https://github.com/Willxup/cpa-usage-keeper/releases)

## DOCUMENTATION

### Overview

CPA Usage Keeper is a standalone observability sidecar for CLIProxyAPI (CPA). It is not a standalone product: it requires a running CLIProxyAPI instance. The service consumes usage events from CPA's Redis queue into a local SQLite database, periodically syncs CPA metadata such as auth files, API keys, and provider info, and serves a built-in React web dashboard for monitoring API consumption.

### Claimed features

- SQLite persistence of CPA usage events consumed from Redis queue.
- Web dashboard for request volume, token usage, cost, cache utilization, success rate, and request performance.
- Filterable request event log by time range, model, API key, source, and outcome.
- Streaming CSV/JSON export.
- Trend analysis, cost breakdowns, provider composition visualization, and hourly activity heatmaps.
- Per-API-key usage queries.
- Credentials management for auth files and AI provider credentials.
- Quota inspection and refresh.
- Model pricing maintenance with manual editing and models.dev sync.
- Latency diagnostics for TTFT / total latency with p95 lines and density rendering.
- Credential health snapshots with 5-hour snapshots and graphs.
- Codex quota cache updates from CPA response headers.
- Optional password-protected access with SHA-256 hashed session tokens persisted in SQLite.
- Automated SQLite backups and configurable data retention window.
- macOS via Homebrew, Linux via binary and systemd, Docker, and Docker Compose support.

## HANDS-ON CHECK

### Overview

Not checked yet.

### Installing

Not checked yet.

### Data access

Not checked yet.

### Verified behavior

Not checked yet.

### Problems

No issues recorded yet.

## OPEN QUESTIONS

- Does CPA Usage Keeper plan to support data sources other than CLIProxyAPI Redis queue?
- Is there a hosted/SaaS version planned?
- What is the retention policy for SQLite data, and is a maximum window documented anywhere?
