# Router-For.ME

## CONCLUSION

### After documentation

- Router-For.ME builds CLIProxyAPI, an open-source proxy server for AI coding CLI tools.
- The core use case is subscription reuse and multi-account pooling through API-compatible endpoints.
- It solves access/routing rather than standalone observability.
- Core still exposes raw per-request token records via `/usage-queue`; this is the only confirmed `usage` capability.
- Built-in aggregate usage dashboard, usage history, and cost tracking were removed from CLIProxyAPI/CPAMC and delegated to companion tools.

### After hands-on

- Installed CLIProxyAPI v7.2.44 macOS arm64 release into `.hands-on/router-for-me/` and ran it with local HOME, config, auth, logs, and temp paths.
- Hands-on confirmed OpenAI-compatible proxying through a local mock provider and confirmed `/usage-queue` emits raw per-request token records.
- Hands-on confirmed legacy aggregate usage endpoints are absent: `/v0/management/usage` and `/v0/management/usage/export` returned 404.
- Hands-on found no built-in cost, usage-history, notification, or hard-stop endpoints in the checked runtime; `/v0/management/cost` and `/v0/management/notifications` returned 404.
- `/api-key-usage` exposes recent success/failed request buckets by provider/API key, not token/cost history and not OAuth subscription analytics.
- `/management.html` and `/management` returned 404 in the isolated binary run even with `disable-control-panel: false`; Management API itself worked.

### Comparison to ai-limits

- Similarity: both are relevant to teams using Claude Code, Codex, Gemini CLI, and similar AI coding tools.
- Similarity: both touch management of multiple AI subscriptions or accounts.
- Difference: CLIProxyAPI routes AI traffic; ai-limits focuses on visibility into actual subscription limit consumption and usage trends.
- Traffic model: CLIProxyAPI puts AI traffic through a proxy; ai-limits is expected to read local session data or telemetry without proxying traffic.
- Privacy model: Router-For.ME adds an extra traffic layer, either self-hosted or through third-party relay services; ai-limits can position around local-only visibility.
- Limit visibility: routing quota state, load balancing, and account management are not counted as ai-limits-style usage/cost capabilities. Hands-on confirmed the remaining built-in telemetry is raw per-request usage records, not an aggregate dashboard, usage history, cost model, forecast, reset countdown, notification system, or hard stop.
- Removed feature signal: usage tracking and logging were explicitly removed in v6.10.0 with the release note "chore: remove usage tracking and logging functionality"; restore requests #3444 and #3481 were closed with `state_reason: not_planned` in May 2026.
- Threat level: Low-Medium ecosystem relevance, but the core Router-For.ME/CLIProxyAPI product is not a direct ai-limits analog after hands-on.
- Bottom line: Router-For.ME is not an ai-limits analog. It only observes requests that users intentionally route through CLIProxyAPI, and it does not track normal local agent usage, provider-side subscription limits, or usage outside the proxy conversion layer.

### What we can learn

- Clear separation between routing and observability can reduce product ambiguity.
- Large ecosystems can delegate specialized usage analytics to companion tools.
- Explicitly removed features are important market signals and should be tracked.

## META

- Date researched: 2026-06-28 (hands-on updated 2026-06-28)
- URL: https://github.com/router-for-me
- Website: https://router-for.me
- Help: https://help.router-for.me
- Relevance: Low-Medium
- Business model: CLIProxyAPI is MIT-licensed and free to self-host. Third-party relay services such as PackyCode, AICodeMirror, and BmoPlus offer paid discounted access. A separate `CLIProxyAPIBusiness` repo exists, but commercial terms are not public.
- License: MIT
- Authors: Unknown
- Owners: Router-For.ME (support@router-for.me)
- GitHub: 38,624 stars on CLIProxyAPI, 6,378 forks (as of 2026-06-28)
- Checked version: CLIProxyAPI v7.2.44, commit 884fc3ce, built 2026-06-28T13:35:59Z
- Latest release: v7.2.44 (2026-06-28), with multiple releases per day noted in the original research
- Founded: Organisation created 2025-09-18
- Sources:
  - [GitHub: router-for-me](https://github.com/router-for-me)
  - [GitHub: CLIProxyAPI](https://github.com/router-for-me/CLIProxyAPI)
  - [CLIProxyAPI README — usage statistics](https://raw.githubusercontent.com/router-for-me/CLIProxyAPI/main/README.md)
  - [CLIProxyAPI releases](https://github.com/router-for-me/CLIProxyAPI/releases)
  - [v6.10.0 release notes — removal of usage tracking](https://github.com/router-for-me/CLIProxyAPI/releases/tag/v6.10.0)
  - [Management API — usage telemetry queue](https://help.router-for.me/management/api)
  - [Issue #3444 — restore usage statistics dashboard](https://github.com/router-for-me/CLIProxyAPI/issues/3444)
  - [Issue #3481 — add usage statistics](https://github.com/router-for-me/CLIProxyAPI/issues/3481)
  - [CLIProxyAPI Docs](https://help.router-for.me/introduction/what-is-cliproxyapi)
  - [DeepWiki: CLIProxyAPI architecture](https://deepwiki.com/router-for-me/CLIProxyAPI)

## DOCUMENTATION

### Overview

Router-For.ME is a GitHub organisation that builds CLIProxyAPI, an open-source proxy server written in Go that exposes AI coding CLI tools such as Claude Code, Codex, Gemini CLI, and Grok Build behind unified OpenAI / Gemini / Claude / Codex-compatible REST API endpoints. The core use case is subscription reuse and multi-account pooling: a developer or team runs CLIProxyAPI locally or on a server, points existing AI coding subscriptions at it, and external tools talk to the proxy as if it were a first-party API.

Architecture documented in the original research:

```text
Client (Cursor / Cline / Continue / any OpenAI client)
        ↓
  CLIProxyAPI (HTTP / WebSocket, Gin framework)
        ↓ OAuth credentials
  AI Providers (Claude Code, Codex, Gemini CLI, Grok Build)
```

Internal layers: Service Coordinator → HTTP API Server → Auth Managers → Model Registry → Provider Executors → Format Translators.

Products documented in the original research:

- CLIProxyAPI: core proxy server in Go.
- Cli-Proxy-API-Management-Center: web UI for CLIProxyAPI configuration and runtime monitoring.
- EasyCLI: desktop GUI for CLIProxyAPI.
- CLIProxyAPIDocs: documentation, also on help.router-for.me via Mintlify.
- CLIProxyAPIBusiness: commercial/business tier with details not public.
- models: model list published by CLIProxyAPI.
- CLIProxyAPI-Plugins-Store: plugin marketplace.
- cpa-plugin-gemini-cli: Gemini CLI plugin.
- cliproxyapi-installer: Linux automated installer.

### Boundary with related tools

This research is about Router-For.ME / CLIProxyAPI itself: the proxy/router that turns existing Claude Code, Codex, Gemini, Grok, and Antigravity access into API-compatible endpoints. Its core responsibility is traffic conversion, credential/account management, load balancing, retry/cooldown behavior, and compatibility translation.

CPA Usage Keeper, CPA-Manager-Plus, CLIProxyAPI Dashboard, and similar projects are related but separate companion tools. They consume CLIProxyAPI traffic, queues, logs, or Management API data and add persistence, dashboards, exports, quota inspection, and cost/usage analytics. Those companion capabilities are not counted as Router-For.ME core capabilities in `_INDEX.yaml`.

The practical distinction matters for ai-limits: CLIProxyAPI can only see traffic that goes through CLIProxyAPI. It does not passively inspect normal Claude Code / Codex / Gemini CLI usage outside the proxy, does not read local agent telemetry files, and does not provide a first-party cross-provider subscription-limit tracker.

### Claimed features

- Wraps Claude Code, ChatGPT Codex, Gemini CLI, Grok Build, and Antigravity behind a single API surface.
- OpenAI / Gemini / Claude / Codex-compatible endpoints for drop-in client use.
- Multi-account load balancing with round-robin and fill-first strategies.
- OAuth-based credential management using existing subscriptions rather than raw API keys.
- Streaming and non-streaming response modes.
- WebSocket support.
- Function calling / tool use.
- Multimodal input with text and images.
- Hot-reload configuration with file watcher, debouncing, and hash verification.
- Plugin architecture in v7.x, including plugin version management, hot reload, plugin logging, and plugin store.
- Multiple storage backends: file, PostgreSQL, Git, and object storage.
- Management API for runtime control.
- Supporting tools include Management Center, EasyCLI, business tier, and Codex Notification tool.

### Usage and cost findings

- Current README states: "Since v6.10.0, CLIProxyAPI and [CPAMC](https://github.com/router-for-me/Cli-Proxy-API-Management-Center) no longer ship built-in usage statistics. If you need usage statistics, use:"
- Current Management API docs state: "Legacy aggregated usage endpoints (`/usage`,`/usage/export`,`/usage/import`) are no longer available. Use `GET /usage-queue` for per-request queue records."
- `/usage-queue` returns raw per-request records with timestamp, latency, source, auth index, token counts, provider, model, endpoint, auth type, API key, and request id. Returned records are removed from the queue.
- `/api-key-usage` returns recent success/failed request buckets grouped by provider and API key; it is not documented as OAuth subscription usage, cost tracking, or quota analytics.
- v6.10.0 release notes include: "chore: remove usage tracking and logging functionality".
- Issue #3444 asked to restore the aggregated usage statistics dashboard for token usage, request counts, and time-series trends; it was closed with `state_reason: not_planned`.
- Issue #3481 asked to add usage statistics back to the web panel; it was closed with `state_reason: not_planned`.
- CPA Usage Keeper, CPA-Manager-Plus, CLIProxyAPI Dashboard, and other ecosystem projects may provide usage/cost dashboards, but they are companion tools and are not counted as Router-For.ME core capabilities.

## HANDS-ON CHECK

### Overview

Installed the macOS arm64 CLIProxyAPI v7.2.44 release into `.hands-on/router-for-me/`, started it on localhost with `-config .hands-on/router-for-me/config/hands-on.yaml`, and tested the Management API plus OpenAI-compatible proxy behavior. The check focused on whether current runtime behavior exposes ai-limits-relevant usage/cost/limit capabilities, especially after the documented removal of built-in usage tracking.

### Installing

Downloaded `CLIProxyAPI_7.2.44_darwin_aarch64.tar.gz` from the GitHub release and extracted it into `.hands-on/router-for-me/bin/`. No package manager or global install was used. Test config, auth directories, HOME, TMPDIR, mock provider script, logs, and downloaded archive were kept under `.hands-on/router-for-me/`.

The binary ran successfully:

```text
CLIProxyAPI Version: 7.2.44, Commit: 884fc3ce, BuiltAt: 2026-06-28T13:35:59Z
```

CLIProxyAPI rewrote the plaintext `remote-management.secret-key` in the hands-on config into a bcrypt hash on startup. This write stayed inside `.hands-on/router-for-me/config/hands-on.yaml`.

### Data access

CLIProxyAPI does not read local Claude Code, Codex, Gemini CLI, or Cursor usage files. It acts as a proxy: usage records are generated from requests that pass through the running server. For the hands-on check, a local OpenAI-compatible mock provider on `127.0.0.1:18318` returned a chat completion with `usage` tokens, and CLIProxyAPI was configured to route `mock-model` to that provider.

The test did not invoke real OAuth login because the core usage/cost question was answerable without touching production accounts: the runtime API itself shows which usage/cost endpoints exist, and a local provider is enough to verify whether `/usage-queue` captures token records.

### Verified behavior

- `GET /v1/models` through CLIProxyAPI returned `mock-model`, confirming the OpenAI-compatible proxy interface works.
- `POST /v1/chat/completions` through CLIProxyAPI returned the mock provider response and usage object.
- `GET /v0/management/usage-statistics-enabled` returned `{"usage-statistics-enabled":true}`.
- `GET /v0/management/usage-queue?count=10` returned a raw per-request record after the chat completion:

```json
{
  "tokens": {
    "input_tokens": 7,
    "output_tokens": 5,
    "reasoning_tokens": 0,
    "cached_tokens": 0,
    "total_tokens": 12
  },
  "provider": "openai-compatible-hands-on-mock",
  "executor_type": "OpenAICompatExecutor",
  "model": "mock-model",
  "endpoint": "POST /v1/chat/completions",
  "auth_type": "apikey"
}
```

- Reading `/usage-queue` popped the record; a second read returned `[]`, confirming queue semantics rather than built-in history/reporting.
- `GET /v0/management/api-key-usage` returned success/failed recent request buckets grouped by provider/API key, with no token totals or cost estimates.
- `GET /v0/management/usage`, `GET /v0/management/usage/export`, `GET /v0/management/cost`, and `GET /v0/management/notifications` returned 404.
- `GET /v0/management/quota-exceeded/switch-project` returned `{"switch-project":true}`; this is routing behavior after quota exhaustion, not a spend/usage hard stop.
- `GET /management.html` and `GET /management` returned 404 in the isolated release run, while `/` and `/v0/management/*` endpoints worked.

### Problems

- Management panel availability differed from README expectations in this isolated run: the binary served API endpoints but did not serve `/management.html` even with `disable-control-panel: false`.
- The config file is mutated on startup when `remote-management.secret-key` is plaintext; this is normal behavior but relevant for isolated testing.
- Runtime confirms a narrow `usage` capability only. It does not confirm `cost`, `usage_history`, `limits`, `session_limits`, `reset`, `burn_rate`, `forecast`, `notifications`, or `hard_stop` for ai-limits comparison.

## OPEN QUESTIONS

- How does CLIProxyAPIBusiness differ from the open-source version, and does it include first-party usage/cost analytics that are absent from CLIProxyAPI?
