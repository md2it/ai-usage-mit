# LLMeter

## CONCLUSION

### After documentation

- LLMeter is an open-source AI cost dashboard for provider billing APIs.
- It targets API spend visibility, not CLI subscription quota tracking.
- It exposes API usage, real cost, historical trends/retention, budget alerts, anomaly alerts, and exports; these are API spend controls, not Claude Code / Codex / Gemini CLI subscription quotas.
- It avoids proxying for providers with billing APIs, and uses SDK wrappers or ingestion for providers/attribution cases where billing APIs are insufficient.
- It explicitly cannot block or throttle API calls in real time.

### After hands-on

- Installed from `amedinat/LLMeter` at commit `8b8b52a` and ran the app locally from `.hands-on/llmeter/`.
- Verified a working production build and local `next start`; public `/` and `/demo` served successfully.
- Verified the public demo dashboard shows API spend, request counts, provider/model breakdowns, daily history, month spend forecast, and optimization suggestions.
- Verified LLMeter is not a fully local tool: the real product experience is an authenticated web app that requires registration/login and external Supabase Auth/PostgreSQL infrastructure.
- Verified authenticated dashboard/API surfaces require Supabase auth; without a session, `/dashboard`, `/providers`, `/api/providers`, `/api/usage/forecast`, and `/api/v1/metrics` redirect to `/login`.
- Verified no CLI subscription quota tracking: there is no local Claude Code / Codex / Gemini CLI data access path, no real quota cap, no remaining credits balance, and no reset countdown.
- Verified Google AI is not connectable in the current UI/code path: provider validators mark `google` as `comingSoonProviders`, and the polling registry excludes it because there is no public usage/billing API.

### Comparison to ai-limits

- Core value: LLMeter is a multi-provider API billing cost dashboard; ai-limits focuses on subscription quota visibility for CLI-based coding tools.
- Data source: LLMeter uses provider usage/billing APIs and an ingestion API; ai-limits is expected to use local CLI session data or telemetry.
- Target user: LLMeter targets developers with direct API spend; ai-limits targets users of Claude Code, Codex, Gemini CLI, and similar subscriptions.
- Locality/dependencies: LLMeter can be self-hosted, but not used as a fully local standalone tool; it needs Supabase Auth/PostgreSQL, user login, provider API keys, and optional Resend/Slack/Paddle integrations.
- CLI subscription awareness: LLMeter does not cover Claude Code, Codex, or Gemini CLI subscription quotas.
- Limits/credits/reset: hands-on confirmed configurable daily/monthly spend alerts and anomaly detection, but no real subscription quota cap, remaining credits balance, or reset countdown for CLI tools.
- Forecast: the app has a month-spend forecast based on historical API spend, but not a forecast of limit-hit time or quota exhaustion.
- Threat level: Low-Medium. The overlap is mostly in multi-provider cost visibility messaging, not in the core data source or usage workflow.

### What we can learn

- Read-only data access is a strong trust signal.
- Clear separation from proxying and SDK changes is valuable positioning.
- Multi-provider spend summaries can create confusion with subscription quota tools, so positioning should be precise.
- Public demo is useful: it shows dashboard value without requiring provider keys or user data.

## META

- Date researched: 2026-06-28
- Hands-on date: 2026-06-28
- URL: https://llmeter.app / https://www.llmeter.org
- GitHub: https://github.com/amedinat/LLMeter
- Checked version: `8b8b52a` (`Develop (#48)`, 2026-06-23)
- Relevance: Low-Medium
- Pricing: Free for 1 provider; Pro $19/mo; Team $49/mo.
- License: AGPL-3.0
- Sources:
  - [GitHub: amedinat/LLMeter](https://github.com/amedinat/LLMeter)
  - [llmeter.app](https://llmeter.app)
  - [llmeter.org](https://www.llmeter.org)
  - [Pricing](https://www.llmeter.org/pricing)
  - [Budget alerts blog](https://www.llmeter.org/blog/llm-api-budget-alerts)

## DOCUMENTATION

### Overview

LLMeter is an open-source AI cost dashboard for API spend monitoring. Its README says it "connects directly to your providers' usage and billing APIs" and "does NOT intercept, proxy, or modify your API calls." For OpenAI, Anthropic, Mistral, DeepSeek, and OpenRouter it documents billing API integrations; for Google AI, Azure OpenAI, and AWS Bedrock it documents SDK wrappers, and its marketing page says Google AI is "coming soon." It is aimed at developers and small teams watching API spend, not CLI-based subscription users.

### Claimed features

- Multi-provider support: OpenAI, Anthropic, DeepSeek, OpenRouter, Mistral, Google AI, Azure OpenAI, and AWS Bedrock, with docs differing on which are billing API integrations versus SDK wrappers.
- Reads from provider billing APIs using read-only keys where available.
- SDK wrappers and ingestion API for providers without billing APIs and for per-customer attribution.
- Budget alerts before surprise invoices; pricing lists one alert on Free and unlimited alerts on Pro/Team.
- Email and Slack webhook notifications for alerts.
- Anomaly detection based on historical spend.
- Real-time cost breakdowns by model, daily trends, usage trends, and CSV/PDF exports.
- Prometheus-compatible metrics endpoint returning `cost_usd`, `requests`, `input_tokens`, and `output_tokens`.
- AES-256-GCM encryption for stored API keys.
- Self-hostable deployment.
- Explicit non-goals: README says LLMeter cannot "Block or throttle API calls in real-time," "Act as a circuit breaker for runaway scripts," or "Modify or intercept your requests to providers."

## HANDS-ON CHECK

### Overview

Installed LLMeter from GitHub into `.hands-on/llmeter/repo` and tested it locally on macOS with Node `v26.0.0`. The goal was to verify the comparison fields from runtime behavior: supported interfaces, providers, capabilities, notifications, hard stop behavior, and especially whether the tool exposes real limits/credits/reset for CLI subscriptions.

### Installing

Installed inside `.hands-on/llmeter/` only:

- Repository: `.hands-on/llmeter/repo`
- Local package manager: `.hands-on/llmeter/pnpm-tools` with `pnpm 11.9.0`
- Isolated caches/store: `.hands-on/llmeter/home`, `.hands-on/llmeter/cache`, `.hands-on/llmeter/npm-cache`, `.hands-on/llmeter/pnpm-store`
- Dependencies: `pnpm install` completed after allowing package build scripts for `sharp`, `msw`, and `unrs-resolver`
- Build: `next build` completed successfully and generated 63 app routes

Installation was not plug-and-play in this environment. `corepack` and global `pnpm` were not available, so a local `pnpm` had to be installed. The first sandboxed install hit a filesystem `EPERM` on pnpm package import; rerunning with full filesystem permission and copy mode fixed it. `pnpm dev` also triggered pnpm's non-interactive dependency check/purge and got stuck after `Recreating node_modules`; running `next` directly avoided that wrapper issue.

### Data access

Hands-on and source review confirmed these data paths:

- Provider polling: encrypted provider API keys stored in Supabase, then background jobs fetch recent usage/cost into `usage_records`
- Initial provider sync: `/api/providers` validates the provider key, stores it encrypted, and immediately fetches the last 30 days of usage
- Dashboard: authenticated pages read `usage_records` from Supabase and aggregate spend by provider, model, and date
- Ingestion API: `/api/ingest` accepts per-call usage events with `model`, `input_tokens`, `output_tokens`, and `customer_id`, authenticated by an LLMeter API key
- Public demo: `/demo` uses a built-in semi-anonymized fixture, not the user's provider account

No local CLI data paths were found or exercised. The app does not read Claude Code, Codex, Gemini CLI, or local JSONL/session files.

### Verified behavior

- `next build` succeeded and listed routes for `/`, `/demo`, `/dashboard`, `/providers`, `/alerts`, `/api/providers`, `/api/ingest`, `/api/usage`, `/api/usage/forecast`, `/api/v1/metrics`, and cron routes.
- `next start` served the production build locally on `http://127.0.0.1:3100`.
- `GET /` returned `200`.
- `GET /demo` returned `200` and rendered a dashboard with `Total Spend (30D)`, `Total Requests`, `Top Provider`, `Month Forecast`, `Daily Spend`, `Usage by Model`, and an optimization card.
- The demo dataset is described in UI as "a real 30-day Anthropic workload (Claude Code + SDK), semi-anonymized"; it displayed Anthropic and OpenAI spend, including model-level rows and request counts.
- `GET /login` returned `200` with Magic Link and Password auth tabs; Google login is disabled as "Google (coming soon)".
- `GET /dashboard`, `GET /providers`, `GET /api/providers`, `GET /api/usage/forecast`, and `GET /api/v1/metrics` returned `307` redirects to `/login?...` without a Supabase-authenticated session.
- `POST /api/ingest` without Bearer returned `401` with `Unauthorized: Missing or invalid API key`; with a dummy Bearer it returned `401` with `Unauthorized: Invalid API key`.
- `GET /api/cron/poll-usage` returned `401` without cron authorization.
- Provider validators list connectable providers as OpenAI, Anthropic, DeepSeek, OpenRouter, Mistral, Azure OpenAI, xAI, Cohere, Groq, Together, Fireworks, Perplexity, Cerebras, AI21, DeepInfra, Novita, Hyperbolic, SambaNova, Lambda Labs, Lepton, Inference.net, NVIDIA NIM, Cloudflare Workers AI, Nebius, and Replicate.
- `google` is present in shared provider types but marked coming soon in the validator and excluded from the polling registry with the comment "Google AI is excluded — no public usage/billing API available."
- Alert code supports `budget_limit`, `daily_threshold`, and `anomaly`; delivery includes email and optional Slack webhook. Test alert endpoint sends a synthetic email, but it requires authentication and configured email service.
- README non-goals are consistent with runtime/source behavior: no request blocking, no throttling, and no circuit breaker.

### Problems

- Self-hosted local usage beyond the public demo requires external Supabase setup. Without a real Supabase project and authenticated session, provider connection, dashboard data, alerts, and v1 metrics cannot be fully exercised.
- In this environment, `next dev` started but produced repeated `EMFILE: too many open files, watch` errors and returned 404 for all tested routes. Production build/start worked.
- `/api/v1/metrics` is documented as Bearer-token API output, but hands-on unauthenticated access was intercepted by middleware and redirected to `/login` before the route's own Bearer auth could run. This may block Prometheus-style scraping unless middleware is bypassed in deployment or a session cookie is also present.
- Documentation and code disagree on provider coverage. README mentions AWS Bedrock SDK wrappers, but no `bedrock` provider type was present in the app types/validators/registry. Google AI is also inconsistent: README lists an SDK wrapper, homepage says coming soon, and runtime validators mark it coming soon.
- The demo mentions "Claude Code + SDK" as workload origin, but the product still treats it as API spend data. It does not expose Claude Code subscription limits, credits, reset, or session windows.

## OPEN QUESTIONS

- Does LLMeter plan to add local-file or CLI session data sources?
- Is there a roadmap for Claude Code / Codex subscription quota tracking?
- Will `/api/v1/metrics` be made public-to-Bearer-auth in middleware so Prometheus can scrape it without a browser session?
- Which provider list is authoritative: homepage/README marketing, runtime `providerTypes`, or polling registry?
