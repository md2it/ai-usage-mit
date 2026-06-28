# Router-For.ME

## CONCLUSION

### After documentation

- Router-For.ME builds CLIProxyAPI, an open-source proxy server for AI coding CLI tools.
- The core use case is subscription reuse and multi-account pooling through API-compatible endpoints.
- It solves access/routing rather than standalone observability.
- Usage tracking was explicitly removed from CLIProxyAPI and delegated to companion tools.

### After hands-on

Not checked yet.

### Comparison to ai-usage-mit

- Similarity: both are relevant to teams using Claude Code, Codex, Gemini CLI, and similar AI coding tools.
- Similarity: both touch management of multiple AI subscriptions or accounts.
- Difference: CLIProxyAPI routes AI traffic; ai-usage-mit focuses on visibility into actual subscription limit consumption and usage trends.
- Traffic model: CLIProxyAPI puts AI traffic through a proxy; ai-usage-mit is expected to read local session data or telemetry without proxying traffic.
- Privacy model: Router-For.ME adds an extra traffic layer, either self-hosted or through third-party relay services; ai-usage-mit can position around local-only visibility.
- Limit visibility: usage tracking and logging were explicitly removed in v6.10.0 with the release note "chore: remove usage tracking and logging functionality"; feature requests were closed as not planned in May 2026 and delegated to CPA Usage Keeper / CPA-Manager-Plus.
- Threat level: Low-Medium. CLIProxyAPI solves a different problem, but its large community creates distribution leverage if the ecosystem expands further into usage monitoring.

### What we can learn

- Clear separation between routing and observability can reduce product ambiguity.
- Large ecosystems can delegate specialized usage analytics to companion tools.
- Explicitly removed features are important market signals and should be tracked.

## META

- Date researched: 2026-06-28 (updated 2026-06-28)
- URL: https://github.com/router-for-me
- Website: https://router-for.me
- Help: https://help.router-for.me
- Relevance: Low-Medium
- Business model: CLIProxyAPI is MIT-licensed and free to self-host. Third-party relay services such as PackyCode, AICodeMirror, and BmoPlus offer paid discounted access. A separate `CLIProxyAPIBusiness` repo exists, but commercial terms are not public.
- License: MIT
- Authors: Unknown
- Owners: Router-For.ME (support@router-for.me)
- GitHub: 38,618 stars on CLIProxyAPI, 6,378 forks (as of 2026-06-28)
- Latest release: v7.2.44 (2026-06-28), with multiple releases per day noted in the original research
- Founded: Organisation created 2025-09-18
- Sources:
  - [GitHub: router-for-me](https://github.com/router-for-me)
  - [GitHub: CLIProxyAPI](https://github.com/router-for-me/CLIProxyAPI)
  - [CLIProxyAPI releases](https://github.com/router-for-me/CLIProxyAPI/releases)
  - [v6.10.0 release notes — removal of usage tracking](https://github.com/router-for-me/CLIProxyAPI/releases/tag/v6.10.0)
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

- Does the Management Center show actual subscription limit consumption, or only proxy throughput metrics?
- Is there a fully local mode with no traffic passing through an external relay?
- How does CLIProxyAPIBusiness differ from the open-source version?
- What is "Antigravity": a new AI provider or internal naming for an existing one?
