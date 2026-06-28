# Router-For.ME

## META

- Date: 2026-06-28
- URL: https://github.com/router-for-me
- Website: https://router-for.me
- Help: https://help.router-for.me
- Relevance: Low-Medium
- Pricing: Open-source (MIT); third-party relay services (PackyCode, AICodeMirror, BmoPlus) offer paid access
- License: MIT
- Authors: Unknown
- Owners: Router-For.ME (support@router-for.me)
- GitHub: 38 600+ stars on CLIProxyAPI, 6 300+ forks (as of 2026-06-28)
- Founded: Organisation created 2025-09-18

## OVERVIEW

Router-For.ME is a GitHub organisation that builds **CLIProxyAPI** — an open-source proxy server written in Go that exposes AI coding CLI tools (Claude Code, Codex, Gemini CLI, Grok Build) behind unified OpenAI / Gemini / Claude / Codex-compatible REST API endpoints.

The core use-case is **subscription reuse and multi-account pooling**: a developer (or team) runs CLIProxyAPI locally or on a server, points their existing Claude Code / Codex / Gemini subscriptions at it, and external tools (Cursor, Cline, Continue, any OpenAI-compatible client) talk to the proxy as if it were a first-party API.

## PRODUCTS

| Repository | Description |
|---|---|
| [CLIProxyAPI](https://github.com/router-for-me/CLIProxyAPI) | Core proxy server. Go. 38.6k stars. |
| [Cli-Proxy-API-Management-Center](https://github.com/router-for-me/Cli-Proxy-API-Management-Center) | Web UI for CLIProxyAPI configuration and runtime monitoring. |
| [EasyCLI](https://github.com/router-for-me/EasyCLI) | Desktop GUI for CLIProxyAPI. |
| [CLIProxyAPIDocs](https://github.com/router-for-me/CLIProxyAPIDocs) | Documentation (also on help.router-for.me via Mintlify). |
| [CLIProxyAPIBusiness](https://github.com/router-for-me/CLIProxyAPIBusiness) | Commercial/business tier (details not public). |
| [models](https://github.com/router-for-me/models) | Model list published by CLIProxyAPI. |
| [CLIProxyAPI-Plugins-Store](https://github.com/router-for-me/CLIProxyAPI-Plugins-Store) | Plugin marketplace. |
| [cpa-plugin-gemini-cli](https://github.com/router-for-me/cpa-plugin-gemini-cli) | Gemini CLI plugin. |
| [cliproxyapi-installer](https://github.com/router-for-me/cliproxyapi-installer) | Linux automated installer. |

## FEATURES

### Core CLIProxyAPI
- Wraps Claude Code, ChatGPT Codex, Gemini CLI, Grok Build, Antigravity behind a single API surface.
- OpenAI / Gemini / Claude / Codex-compatible endpoints — drop-in for any client.
- **Multi-account load balancing**: round-robin and fill-first strategies; distributes requests across multiple OAuth sessions to avoid rate limits.
- OAuth-based credential management (no raw API keys needed — uses existing subscriptions).
- Streaming and non-streaming response modes, WebSocket support.
- Function calling / tool use, multimodal input (text + images).
- **Hot-reload configuration** — file watcher with debouncing + hash verification, no service restart required.
- Plugin architecture for extensibility.
- Multiple storage backends: file, PostgreSQL, Git, object storage.
- Management API for runtime control.

### Supporting Tools
- Web UI (Management Center) for config changes and runtime monitoring.
- Desktop GUI (EasyCLI).
- Business tier (CLIProxyAPIBusiness) with presumably additional features.
- Codex Notification tool.

## ARCHITECTURE

```
Client (Cursor / Cline / Continue / any OpenAI client)
        ↓
  CLIProxyAPI (HTTP / WebSocket, Gin framework)
        ↓ OAuth credentials
  AI Providers (Claude Code, Codex, Gemini CLI, Grok Build)
```

Internal layers: Service Coordinator → HTTP API Server → Auth Managers → Model Registry → Provider Executors → Format Translators.

## PRICING / BUSINESS MODEL

- **CLIProxyAPI itself is MIT-licensed and free to self-host.**
- Sponsor integrations with third-party relay services (PackyCode, AICodeMirror, BmoPlus) offer paid discounted access as an alternative to self-hosting.
- A separate `CLIProxyAPIBusiness` repo exists; commercial terms are not public.

## COMPARISON WITH AI-USAGE-MIT

### Similar

- Both products are relevant to teams using Claude Code, Codex, Gemini CLI and similar AI coding tools.
- Both touch the concept of managing multiple AI subscriptions / accounts.
- Management Center provides runtime monitoring — surface overlap with usage dashboards.

### Difference

| Dimension | Router-For.ME (CLIProxyAPI) | ai-usage-mit |
|---|---|---|
| **Core value** | Proxy — routes traffic to AI providers; enables reuse of CLI subscriptions via API | Visibility — shows actual subscription limit consumption and usage trends |
| **Traffic model** | All AI traffic passes through the proxy | No traffic proxying; reads local session data / telemetry |
| **Privacy** | User AI traffic routed through an extra layer (self-hosted or third-party) | Fully local, no traffic interception |
| **Target user** | Developers who want to use CLI subscriptions in non-CLI tools (Cursor, Cline) | Developers who want to understand how much of their quota they have consumed |
| **Limit visibility** | Explicitly removed: since v6.10.0 *"CLIProxyAPI and CPAMC no longer ship built-in usage statistics"* ([README](https://github.com/router-for-me/CLIProxyAPI/blob/main/README.md)); delegated to third-party tools (CPA Usage Keeper, CPA-Manager-Plus) | Core feature |
| **Multi-account pooling** | Yes (round-robin across OAuth sessions) | Not applicable |
| **License** | MIT (open-source) | — |

### Threat level

**Low-Medium.** CLIProxyAPI solves a different problem (access / routing) rather than observability. However, the Management Center could expand into usage monitoring over time, and the large community (38k stars) gives them significant distribution leverage if they pivot.

## OPEN QUESTIONS

- Does the Management Center show actual subscription limit consumption, or only proxy throughput metrics?
- Is there a fully local mode with no traffic passing through an external relay?
- How does CLIProxyAPIBusiness differ from the open-source version?
- What is the growth trajectory — are they expanding into analytics / observability?

## SOURCES

- [GitHub: router-for-me](https://github.com/router-for-me)
- [GitHub: CLIProxyAPI](https://github.com/router-for-me/CLIProxyAPI)
- [CLIProxyAPI Docs](https://help.router-for.me/introduction/what-is-cliproxyapi)
- [DeepWiki: CLIProxyAPI architecture](https://deepwiki.com/router-for-me/CLIProxyAPI)
