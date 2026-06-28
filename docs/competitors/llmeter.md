# LLMeter

## META

- Date researched: 2026-06-28
- URL: https://llmeter.app / https://www.llmeter.org
- GitHub: https://github.com/amedinat/LLMeter
- Relevance: Low-Medium
- Pricing: Free (1 provider on hosted); self-host for unlimited
- License: Open-source (self-hostable)

## OVERVIEW

LLMeter is an open-source AI cost dashboard that pulls usage data directly from provider billing APIs (OpenAI, Anthropic, DeepSeek, OpenRouter, Google AI, Mistral) using read-only API keys. No proxy, no SDK, no code changes — paste an API key and the dashboard populates with spend by model, daily cost trends, and breakdowns. Targets developers and small teams watching API spend. Not designed for CLI-based subscription users (Claude Code, Codex, Gemini CLI).

## FEATURES

- Multi-provider: OpenAI, Anthropic, Google AI, DeepSeek, OpenRouter, Mistral
- Reads from provider billing APIs using read-only keys
- Budget alerts before surprise invoices
- Real-time cost breakdowns by model
- AES-256-GCM encryption for stored API keys
- Self-hostable

## PRICING / BUSINESS MODEL

Free for 1 provider on the hosted version. Self-host for unlimited providers. No paid tiers described publicly.

## COMPARISON WITH AI-USAGE-MIT

| Dimension | LLMeter | ai-usage-mit |
|---|---|---|
| **Core value** | Multi-provider API billing cost dashboard | Subscription quota visibility for CLI-based coding tools |
| **Data source** | Provider billing APIs (read-only API keys) | Local CLI session data / telemetry |
| **Target user** | Developers with direct API spend to watch | Developers with Claude Code / Codex / Gemini CLI subscriptions |
| **Claude Code / Codex / Gemini CLI awareness** | No — billing APIs only, not CLI subscription quotas | Core feature |
| **Quota / limit tracking** | No — cannot track Pro/Max subscription consumption | Core feature |
| **Burn-rate prediction** | No | — |
| **Per-session / per-project granularity** | No | — |

### Threat level

**Low-Medium.** Different data source (billing APIs vs. local files), different audience (API cost watchers vs. subscription CLI users). No awareness of Claude Code / Codex / Gemini CLI workflows. Overlaps only in "multi-provider cost visibility" messaging, not in the core mechanic. Risk is mainly positioning confusion, not direct feature competition.

## OPEN QUESTIONS

- Does LLMeter plan to add local-file / CLI session data sources?
- Is there a roadmap for Claude Code / Codex subscription quota tracking?

## SOURCES

- [GitHub: amedinat/LLMeter](https://github.com/amedinat/LLMeter)
- [llmeter.app](https://llmeter.app)
