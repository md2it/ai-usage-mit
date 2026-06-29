# CPA-Manager-Plus

## CONCLUSION

### After documentation

- CPA-Manager-Plus is a self-hosted management and analytics dashboard built on top of CLIProxyAPI.
- It is an ops panel rather than a standalone AI usage product.
- Source review confirms monitoring, usage history, pricebook-based cost/token analytics, Codex quota windows, and CPA auth-file limit controls for CLIProxyAPI users.
- Codex quota inspection is not a standalone Codex CLI quota reader: it depends on CLIProxyAPI management APIs and CPA `api-call`.
- It appears actively developed, with the last documented release on June 26, 2026.

### After hands-on

- Native macOS arm64 release `v1.9.1` installed and run from `.hands-on/cpa-manager-plus/`.
- Manager Server started successfully on `127.0.0.1:18327` with `USAGE_DATA_DIR`, `USAGE_DB_PATH`, and `CPA_MANAGER_DATA_KEY_PATH` inside the hands-on directory.
- Setup against a live local CLIProxyAPI at `127.0.0.1:18317` succeeded with HTTP collector mode.
- A controlled request through CPA to an OpenAI-compatible local provider was ingested into CPAMP SQLite and exposed through usage, dashboard, monitoring, header snapshot, and export APIs.
- Dashboard and monitoring runtime output confirmed usage history, token totals, latency, rolling 30-minute burn-rate (`rpm`/`tpm`), and model/channel health views.
- Cost calculation path was visible in dashboard/analytics responses, but the controlled `mock-model` had no price entry, so hands-on confirmed zero-cost output rather than a non-zero real billing estimate.
- Codex inspection run executed and completed through CPAMP, but the connected CPA had no Codex auth files; therefore real Codex quota windows and quota reset values were not runtime-verified.
- The hands-on check reinforced that Codex quota inspection is not standalone: without CPA auth files, CPAMP can only complete an empty inspection run.

### Comparison to ai-limits

- Goal: CPA-Manager-Plus is an ops panel and analytics layer for CLIProxyAPI; ai-limits is expected to focus on AI usage tracking.
- Deployment: CPA-Manager-Plus is self-hosted through Docker or native binary.
- Data source: CLIProxyAPI usage queue / HTTP usage queue / RESP fallback, plus Codex quota probing through CPA `api-call`.
- Cost analytics: documented by model, provider, account, API key, project, channel, and time window.
- Quota analytics: Codex windows can come from ChatGPT `/backend-api/wham/usage` via CPA, or from observed response headers stored with usage events.
- Provider support: usage analytics are CPA-wide; Codex has the deepest explicit quota/inspection path.
- Pricing: free / MIT.
- Target audience: CPA self-hosters, with strong relevance to the Chinese self-hosting community noted in the original research.
- Tracking boundary: CPA-Manager-Plus only sees usage that is converted into API traffic and routed through CLIProxyAPI. Direct provider usage, local CLI usage, and agent activity outside CPA are invisible to it.
- Reference relevance: Threat level is low. CPA-Manager-Plus is a good CPA tool, but absolutely irrelevant for ai-limits unless users intentionally proxy ordinary provider/agent usage through CLIProxyAPI.

### What we can learn

- Response-header observability for quota windows is a useful data surface where available.
- Privacy masking for screenshots is a practical feature for sharing usage dashboards.
- Plugin ecosystems can extend usage tooling, but they also increase operational complexity.

## META

- Date researched: 2026-06-28
- URL: https://github.com/seakee/CPA-Manager-Plus
- Relevance: Management panel + usage analytics companion for CLIProxyAPI
- Pricing: Free (MIT license, no SaaS tier)
- License: MIT
- Stars / Forks: 945 / not checked via clone metadata
- Last release: v1.9.1 — June 26, 2026
- Stack: TypeScript/React 19 frontend, Go 1.24 backend, SQLite
- Sources:
  - [GitHub repository](https://github.com/seakee/CPA-Manager-Plus)
  - [README](https://github.com/seakee/CPA-Manager-Plus#readme)
  - [Wiki](https://github.com/seakee/CPA-Manager-Plus/wiki)
  - [Releases / Changelog](https://github.com/seakee/CPA-Manager-Plus/releases)
  - [CLIProxyAPI upstream](https://github.com/router-for-me/CLIProxyAPI)
  - Local source review in `.hands-on/cpa-manager-plus/repo` at `main` shallow clone on 2026-06-28
  - Local hands-on run of native release `cpa-manager-plus_v1.9.1_darwin_arm64.tar.gz` on 2026-06-28

## DOCUMENTATION

### Overview

CPA-Manager-Plus is a self-hosted management and analytics dashboard built on top of CLIProxyAPI (CPA). It is not an AI product by itself: it is an ops panel that converts the CPA usage queue into a SQLite request log and provides monitoring, cost/token analytics, quota views, account management, and plugin integration. It targets self-hosters who already run CLIProxyAPI and want visibility into spend, request health, and quota state. It does not analyze provider or agent usage that bypasses CLIProxyAPI.

Components documented in the original research:

- Manager Server on port 18317: Go backend with monitoring, analytics, and storage.
- CPA Panel on port 8317: lightweight panel without monitoring and analytics; only CPA management.
- Docker image: `seakee/cpa-manager-plus` on Docker Hub and `ghcr.io/seakee/cpa-manager-plus`.

### Claimed features

- SQLite request ledger: "Converts CPA usage queue into SQLite request ledger for live monitoring, historical search, import/export, and analytics" (README).
- Cost and token analytics by model, provider, account, API key, project, channel, and time window.
- Response header observability in v1.9.0 for "current window, usage rate, recovery time, and plan type" from Codex quota headers.
- Codex account inspection for quota window, OAuth token status, and quota masking for screenshots.
- Model pricing sync from LiteLLM and OpenRouter.
- Plugin ecosystem introduced in v1.5.0: marketplace, management, and OAuth-provider support.
- Account pool safeguards, including auto-recovery of disabled auth files after quota reset.
- Privacy masking in v1.8.1 for sharing screenshots.
- Admin-key authentication and encrypted storage of the CPA management key.
- Cross-platform packaging for Linux, macOS, and Windows on amd64 and arm64.

### Source-reviewed findings

- Usage history is backed by a SQLite `usage_events` table with request id, timestamp, provider, model, auth/account snapshots, token buckets, latency, failure fields, and response-header quota metadata (`apps/manager-server/internal/repository/sqlite/migrate.go`).
- Usage analytics aggregate calls, success/failure counts, input/output/reasoning/cache tokens, timelines, heatmaps, model/API-key/credential/channel stats, and event pages from `usage_events` (`apps/manager-server/internal/repository/usageevent/aggregate.go`, `apps/manager-server/internal/repository/usageevent/analytics.go`).
- Cost is estimated from token aggregates and stored model prices, not directly from an upstream billing ledger (`apps/manager-server/internal/service/pricing/cost.go`, `apps/manager-server/internal/service/monitoring/service.go`).
- Burn-rate is exposed as rolling 30-minute request/token velocity (`rpm_30m`, `tpm_30m`) in monitoring analytics and dashboard summaries (`apps/manager-server/internal/service/monitoring/service.go`, `apps/manager-server/internal/service/dashboard/service.go`).
- Codex quota views fetch ChatGPT usage data through CPA `api-call`: frontend `fetchCodexQuota` sends CPA `authIndex` and GETs `https://chatgpt.com/backend-api/wham/usage` via `apiCallApi.request` (`apps/web/src/utils/quota/providerRequests.ts`).
- Codex response-header quota fallback is parsed from real CPA usage event response headers; v1.9.0 release notes explicitly say Manager Server "parses, persists, and backfills quota, error, trace, and routing metadata from response headers" and derives "current windows, usage percentage, recovery time, plan type, and rate-limit window source from Codex quota headers".
- Codex quota inspection is a CPA-based account-health probe, not a standalone Codex quota inspection feature. Backend `codexinspection.Service` requires configured `CPAConnection.CPABaseURL` and `ManagementKey`, lists `/v0/management/auth-files`, then calls CPA `/v0/management/api-call` to request `https://chatgpt.com/backend-api/wham/usage` (`apps/manager-server/internal/service/codexinspection/service.go`).
- Limit control is implemented through CPA auth-file actions: Codex inspection can suggest/execute `disable`, `enable`, or `delete`, and the near-real-time rate-limit worker disables CPA auth files after Codex 429 usage-limit events with an explicit reset time, then re-enables CPAMP-owned cooldowns later (`apps/manager-server/internal/service/codexinspection/service.go`, `apps/manager-server/internal/worker/rate_limit_auto_disable.go`).
- Credits are ambiguous for the `_INDEX.yaml` closed vocabulary. The code parses Codex `credits`/`credits_balance` and displays rate-limit reset credits, but source review did not confirm a primary UI surface for a standalone prepaid/API credit balance. Therefore `credits` is not listed in `compare.capabilities`.

## HANDS-ON CHECK

### Overview

CPA-Manager-Plus `v1.9.1` was installed from the official `darwin_arm64` native release and run as Manager Server mode from `.hands-on/cpa-manager-plus/`. It was connected to an already running local CLIProxyAPI instance from `.hands-on/router-for-me/` at `http://127.0.0.1:18317`, using the CPA management key from that isolated hands-on config. The check verified startup, setup, usage ingestion, analytics, export, and Codex inspection behavior against a live CPA process.

### Installing

- Install path: `.hands-on/cpa-manager-plus/bin/cpa-manager-plus`.
- Source archive: `.hands-on/cpa-manager-plus/downloads/cpa-manager-plus_v1.9.1_darwin_arm64.tar.gz`.
- Checksum observed: `f03ab72fd11c27690b52a0f2809ba2691cd132695e267972cfe66b7ccb59b67d`.
- Runtime data path: `.hands-on/cpa-manager-plus/data/`.
- The native binary ran without Go, Node, or Docker. Docker was not usable because Docker daemon was not running; Go was not installed in the shell PATH.
- Default port `18317` was already used by the local `cli-proxy` process, so CPAMP was started on `127.0.0.1:18327`.

### Data access

- Setup API accepted `cpaBaseUrl: http://127.0.0.1:18317`, `cpaManagementKey: hands-on-management-key`, and `collectorMode: http`.
- CPAMP read CPA management config and reported `usageStatisticsEnabled: true`, `redisUsageQueueRetentionSeconds: 600`.
- The connected CPA had no auth files: `/v0/management/auth-files` returned `{"files":[]}`.
- The connected CPA exposed one OpenAI-compatible local model, `mock-model`, through `hands-on-mock`.
- One controlled request through CPA to `/v1/chat/completions` produced one usage event; CPAMP collected it into `.hands-on/cpa-manager-plus/data/usage.sqlite`.
- Requests that do not pass through CPA are outside CPAMP's data model; the tool does not import local agent histories or provider billing ledgers directly.

### Verified behavior

- Health and setup:
  - `/health` returned `{"ok":true,"service":"cpa-manager-plus"}`.
  - `/usage-service/info` moved from `setupRequired: true` to `configured: true`, `projectInitialized: true`.
- Usage history:
  - `/v0/management/usage` returned `total_requests: 1`, `success_count: 1`, `total_tokens: 12`.
  - `/v0/management/usage/export` returned NDJSON with request id, event hash, timestamp, provider, model, endpoint, token buckets, latency, TTFT, API key hash, auth index, and response metadata.
- Dashboard:
  - `/v0/management/dashboard/summary` returned today's totals, `top_models_today`, `model_cost_rank`, traffic timeline, token mix, and channel health.
  - Rolling burn-rate was present as `rpm: 0.03333333333333333` and `tpm: 0.4` for one request / 12 tokens in the 30-minute window.
- Monitoring analytics:
  - `/v0/management/monitoring/analytics` returned timeline and event page rows with model, endpoint, auth/source hashes, token counts, latency, TTFT, failure status, and response metadata.
  - `/v0/management/monitoring/header-snapshots` returned the collected response metadata for the event.
- Cost:
  - Dashboard and monitoring responses included `cost` fields.
  - The controlled `mock-model` had no price entry, so runtime cost was `0`; source review remains the evidence that real cost is pricebook-based estimation.
- Codex inspection:
  - Manual `POST /v0/management/codex-inspection/run` completed a run with `totalFiles: 0`, `probeSetCount: 0`, and empty `results`.
  - This confirms the runtime path is CPA auth-file based. It did not verify real Codex quota windows because the connected CPA had no Codex auth files.
- Limit controls:
  - `/usage-service/account-processing-policy` showed quota cooldown and account-action auto-disable settings present but disabled in this run.
  - `/usage-service/quota-cooldowns` returned an empty list.

### Problems

- Default CPAMP port `18317` conflicted with the already running local CLIProxyAPI, so CPAMP needed a different port.
- Docker quick start could not be used because Docker daemon was unavailable; native release worked without issue.
- The connected CPA had no Codex auth files, so real Codex quota windows, reset timestamps, credits, and account enable/disable actions were not verified at runtime.
- The available local provider was a controlled OpenAI-compatible mock, not a production provider account. It was enough to verify CPA usage ingestion and analytics, but not enough to verify non-zero real cost estimates.
- No user-facing notification capability was confirmed in hands-on; `_INDEX.yaml` keeps `notifications: null`.
- README-level quota/cost claims were not accepted as facts by themselves; runtime findings are separated from source-reviewed findings above.

## OPEN QUESTIONS

- What is the exact scope of ai-limits? If it overlaps with SQLite analytics for CPA users, the reference relevance is higher.
- Do the authors of CPA-Manager-Plus plan to support sources beyond CLIProxyAPI?
- How large is the Chinese self-hosting community as a target market for ai-limits?
- Does CPA-Manager-Plus intentionally expose Codex credit balance as a first-class UI capability, or only retain it as response-header metadata / search context?
