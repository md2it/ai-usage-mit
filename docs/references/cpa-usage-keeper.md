# CPA Usage Keeper

## CONCLUSION

### After documentation

- CPA Usage Keeper is a self-hosted observability sidecar for CLIProxyAPI (CPA), not a direct multi-provider tracker.
- It requires a running CLIProxyAPI instance; `CPA_BASE_URL` and `CPA_MANAGEMENT_KEY` are mandatory.
- It consumes CPA's Redis usage queue into SQLite, then exposes historical CPA event analytics through a React dashboard and aggregation APIs.
- Local SQLite/dashboard views cover request/token history, cost estimates, latency diagnostics, trends, filtering, and CSV/JSON export.
- Live CPA/Redis is still required for queue ingestion, metadata sync, credential health, quota-window refresh/inspection, and reset timing.
- It is actively developed and has a broad usage analytics feature set for CPA operators, but provider coverage should be read as CPA event visibility rather than direct provider support.

### After hands-on

- Installed v1.12.1 macOS arm64 release inside `.hands-on/cpa-usage-keeper/`; binary install is simple and self-contained.
- Keeper starts successfully with `-env`, creates SQLite under configured `WORK_DIR`, serves a React dashboard on the configured port, and exposes health/status/version/usage/pricing/quota APIs.
- Without a live CPA endpoint, dashboard/API still start, but Redis ingestion and metadata sync report connection errors and all usage views remain empty.
- With an isolated CLIProxyAPI v7.2.44 instance, Keeper connected to the CPA Redis/RESP endpoint, selected the `usage` key, synced CPA API-key/provider identity metadata, and switched metadata sync to notification mode.
- End-to-end usage ingestion was verified with an isolated OpenAI-compatible mock provider through CPA: one successful request was imported into SQLite and appeared in overview, event log, identity stats, JSON export, and CSV export.
- Cost calculation was verified after adding a local pricebook entry for the mock model; the same imported event changed from `cost_available:false` to a computed USD cost.
- CPA Usage Keeper only tracks requests that were routed through CLIProxyAPI and became CPA/API traffic. Direct provider usage, local CLI usage, and agent activity outside CPA are invisible to it.
- Quota/limit/reset behavior was not verified against real provider credentials. In the mock OpenAI-compatible identity, quota cache returned no rows and quota refresh rejected the identity as `not_auth_file`.
- No existing live CPA with real auth entries was found during the check, so hands-on did not validate real Claude/Codex/Gemini quota windows or reset timing.

### Comparison to ai-limits

- Standalone operation: CPA Usage Keeper requires CLIProxyAPI; ai-limits is expected to operate more broadly.
- Data source: CPA Usage Keeper reads from the CLIProxyAPI Redis queue only.
- Storage: CPA Usage Keeper uses local SQLite.
- Dashboard: CPA Usage Keeper has a built-in React web UI.
- Pricing visibility: CPA Usage Keeper tracks cost and can use models.dev pricing.
- Credential health: CPA Usage Keeper includes credential health snapshots, but hands-on only verified empty/mock identity states.
- Export: CSV/JSON export is documented.
- Threat level: Low for ai-limits. It is a good CPA sidecar, but absolutely irrelevant for tracking user spend/usage that happens directly in providers or agents and never passes through CLIProxyAPI.

### What we can learn

- A local SQLite event ledger is a practical model for searchable usage history.
- Filterable request logs and CSV/JSON export are useful for power users.
- Credential health and quota snapshots are relevant adjacent workflows for users managing multiple AI accounts.

## META

- Date: 2026-06-28
- URL: https://github.com/Willxup/cpa-usage-keeper
- Package checked hands-on: `cpa-usage-keeper_v1.12.1_darwin_arm64.tar.gz`
- Checked version: v1.12.1
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

CPA Usage Keeper is a self-hosted observability sidecar for CLIProxyAPI (CPA). It is not a standalone multi-provider product: it requires a running CLIProxyAPI instance. The service consumes usage events from CPA's Redis queue into a local SQLite database, periodically syncs CPA metadata such as auth files, API keys, and provider info, and serves a built-in React web dashboard for monitoring API consumption. Its provider visibility comes through CPA usage events and CPA management calls, not direct integrations with upstream providers.

### Claimed features

- SQLite persistence of CPA usage events consumed from Redis queue.
- Web dashboard for request volume, token usage, cost, cache utilization, success rate, and request performance.
- Filterable request event log by time range, model, API key, source, and outcome.
- Streaming CSV/JSON export.
- Trend analysis, cost breakdowns, provider composition visualization, and hourly activity heatmaps.
- Per-API-key usage queries.
- Credentials management for auth files and AI provider credentials.
- Quota inspection and refresh through live CPA management calls.
- Model pricing maintenance with manual editing and models.dev sync.
- Latency diagnostics for TTFT / total latency with p95 lines and density rendering.
- Credential health snapshots with 5-hour snapshots and graphs.
- Provider quota-window rows can include used/limit/remaining, reset timing, and window token/cost fields, based on live quota refresh/cache data.
- Codex quota cache updates from CPA response headers.
- Optional password-protected access with SHA-256 hashed session tokens persisted in SQLite.
- Automated SQLite backups and configurable data retention window.
- macOS via Homebrew, Linux via binary and systemd, Docker, and Docker Compose support.

### Capability split

- Local SQLite/dashboard: historical CPA request events, token totals, cost estimates, latency diagnostics, trend/heatmap analysis, per-API-key usage queries, request-event filtering, and CSV/JSON export.
- Requires live CPA/Redis: new event ingestion from the CPA Redis queue, CPA metadata sync, credential health/quota refresh and inspection, provider quota-window display, reset timing, and Codex quota cache updates from CPA response headers.
- Not confirmed from docs/runtime: notifications, enforced usage/spend stopping, forecasts, or burn-rate tracking.

## HANDS-ON CHECK

### Overview

Installed and ran CPA Usage Keeper v1.12.1 from the macOS arm64 release archive in `.hands-on/cpa-usage-keeper/`. The check first started Keeper against an absent CPA endpoint to observe failure behavior, then ran an isolated CLIProxyAPI v7.2.44 on localhost with `usage-statistics-enabled: true`, and finally sent a request through a local mock OpenAI-compatible provider to verify the ingestion path without calling external providers.

### Installing

Install path: `.hands-on/cpa-usage-keeper/`. The Keeper release archive contains `cpa-usage-keeper`, `.env.example`, README files, license, and an empty `data/` directory. The binary supports `-env path`; no Homebrew, global service, Docker container, or files outside the hands-on workspace were needed for Keeper itself.

For the live dependency check, CLIProxyAPI v7.2.44 was also downloaded into `.hands-on/cpa-usage-keeper/cpa/`. Running CPA non-interactively required `-standalone`; without it, the binary printed that the API server started and then exited. The isolated CPA config bound to `127.0.0.1:18317`, used an auth directory inside hands-on, enabled `usage-statistics-enabled: true`, and exposed a local-only management key used by Keeper.

### Data access

Keeper does not read Claude/Codex/Gemini local files directly and does not inspect arbitrary agent activity. It reads CPA management APIs and the CPA Redis/RESP usage stream, so usage must first be converted into requests routed through CLIProxyAPI. In the successful isolated run, Keeper logs showed `subscribe_connected`, selected Redis key `usage`, performed a zero-message backfill, then received usage through CPA after a mock provider request. SQLite files were created under `.hands-on/cpa-usage-keeper/data/`.

Local SQLite/dashboard features verified: request/tokens overview, event list, source identity stats, service health success rate, pricebook-based cost calculation, JSON export, and CSV export.

Live CPA/Redis features verified: connection to CPA Redis/RESP usage stream, CPA API-key metadata sync, OpenAI-compatible provider identity sync, and ingestion from CPA usage events into SQLite.

Provider credential features not verified: auth-file quota windows, real subscription/session limits, reset timing, credential health for Claude/Codex/Gemini accounts, and quota reset actions. The mock OpenAI-compatible identity was not an auth-file identity, so `/api/v1/quota/refresh` rejected it as `not_auth_file`.

### Verified behavior

- `GET /healthz` returned `{"status":"ok"}` and `GET /api/v1/version` returned v1.12.1.
- With no CPA on `127.0.0.1:8317`, Keeper stayed up but status contained Redis/management connection failures and all usage APIs returned empty data.
- With isolated CPA running on `127.0.0.1:18317`, Keeper status returned `running:true`, `sync_running:true`, and `last_status:"empty"` before test traffic.
- A test request through CPA to the local mock provider produced a CPA usage event with 11 input tokens, 7 output tokens, and 18 total tokens.
- Keeper imported that event into SQLite: `usage_events|1`, `usage_identities|1`, `cpa_api_keys|1`, `redis_usage_inboxes|1`.
- `GET /api/v1/usage/overview?range=24h` showed one request, 18 tokens, 100% success rate, RPM/TPM series, and after pricing was added, `total_cost:0.000024999999999999998`.
- `GET /api/v1/usage/events?range=24h&page_size=20` showed the imported request with redacted CPA API key, model, endpoint, source identity, token breakdown, latency, and computed cost.
- `GET /api/v1/usage/identities` showed the synced mock identity and aggregated request/token counters.
- JSON and CSV exports returned the same event and token/cost fields.
- `POST /api/v1/quota/cache` returned no quota rows for the mock identity; `POST /api/v1/quota/refresh` rejected it with `not_auth_file`.

### Problems

- A real CPA instance was not already running on the default `127.0.0.1:8317`.
- No `.cli-proxy-api` configuration was found in accessible local paths, and Docker was unavailable, so hands-on could not use real CPA provider/auth-file data.
- CLIProxyAPI needs `-standalone` in this non-interactive hands-on setup; otherwise it starts and exits cleanly.
- Reading CPA `/v0/management/usage-queue` manually can consume queued events before Keeper imports them. A second request, without manual queue polling, was imported by Keeper.
- Quota/limit/reset capabilities could not be validated with real Claude/Codex/Gemini auth-file credentials; only the negative mock-provider behavior was verified.

## OPEN QUESTIONS

- Does CPA Usage Keeper plan to support data sources other than CLIProxyAPI Redis queue?
- Is there a hosted/SaaS version planned?
- How does quota-window display behave hands-on with real Claude/Codex/Gemini CPA auth-file credentials?
