# CPA-Manager-Plus

## CONCLUSION

### After documentation

- CPA-Manager-Plus is a self-hosted management and analytics dashboard built on top of CLIProxyAPI.
- It is an ops panel rather than a standalone AI usage product.
- It provides monitoring, cost/token analytics, account management, and plugin integration for CLIProxyAPI users.
- It appears actively developed, with the last documented release on June 26, 2026.

### After hands-on

Not checked yet.

### Comparison to ai-usage-mit

- Goal: CPA-Manager-Plus is an ops panel and analytics layer for CLIProxyAPI; ai-usage-mit is expected to focus on AI usage tracking.
- Deployment: CPA-Manager-Plus is self-hosted through Docker or native binary.
- Data source: CLIProxyAPI / Codex through CPA.
- Cost analytics: documented by model, provider, account, API key, project, channel, and time window.
- Provider support: Claude Code, Codex, and Gemini CLI are available through CPA.
- Pricing: free / MIT.
- Target audience: CPA self-hosters, with strong relevance to the Chinese self-hosting community noted in the original research.
- Reference relevance: Low. CPA-Manager-Plus is tightly coupled to CLIProxyAPI. If ai-usage-mit targets broader or agent-level usage tracking outside CLIProxyAPI, direct overlap is limited.

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
- Stars / Forks: 944 / 113
- Last release: v1.9.1 — June 26, 2026
- Stack: TypeScript/React 19 frontend, Go 1.24 backend, SQLite
- Sources:
  - [GitHub репозиторий](https://github.com/seakee/CPA-Manager-Plus)
  - [README](https://github.com/seakee/CPA-Manager-Plus#readme)
  - [Wiki](https://github.com/seakee/CPA-Manager-Plus/wiki)
  - [Releases / Changelog](https://github.com/seakee/CPA-Manager-Plus/releases)
  - [CLIProxyAPI upstream](https://github.com/router-for-me/CLIProxyAPI)

## DOCUMENTATION

### Overview

CPA-Manager-Plus is a self-hosted management and analytics dashboard built on top of CLIProxyAPI (CPA). It is not an AI product by itself: it is an ops panel that converts the CPA usage queue into a SQLite request log and provides monitoring, cost/token analytics, account management, and plugin integration. It targets self-hosters who already run CLIProxyAPI and want visibility into spend and quotas.

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

- What is the exact scope of ai-usage-mit? If it overlaps with SQLite analytics for CPA users, the reference relevance is higher.
- Do the authors of CPA-Manager-Plus plan to support sources beyond CLIProxyAPI?
- How large is the Chinese self-hosting community as a target market for ai-usage-mit?
