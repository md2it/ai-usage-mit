# LLMeter

## CONCLUSION

### After documentation

- LLMeter is an open-source AI cost dashboard for provider billing APIs.
- It targets API spend visibility, not CLI subscription quota tracking.
- It avoids proxying and SDK changes by pulling usage data directly from provider billing APIs with read-only keys.

### After hands-on

Not checked yet.

### Comparison to ai-usage-mit

- Core value: LLMeter is a multi-provider API billing cost dashboard; ai-usage-mit focuses on subscription quota visibility for CLI-based coding tools.
- Data source: LLMeter uses provider billing APIs; ai-usage-mit is expected to use local CLI session data or telemetry.
- Target user: LLMeter targets developers with direct API spend; ai-usage-mit targets users of Claude Code, Codex, Gemini CLI, and similar subscriptions.
- CLI subscription awareness: LLMeter does not cover Claude Code, Codex, or Gemini CLI subscription quotas.
- Quota and burn-rate tracking: not documented for CLI subscriptions.
- Threat level: Low-Medium. The overlap is mostly in multi-provider cost visibility messaging, not in the core data source or usage workflow.

### What we can learn

- Read-only data access is a strong trust signal.
- Clear separation from proxying and SDK changes is valuable positioning.
- Multi-provider spend summaries can create confusion with subscription quota tools, so positioning should be precise.

## META

- Date researched: 2026-06-28
- URL: https://llmeter.app / https://www.llmeter.org
- GitHub: https://github.com/amedinat/LLMeter
- Relevance: Low-Medium
- Pricing: Free for 1 provider on the hosted version; self-host for unlimited providers. No paid tiers described publicly.
- License: Open-source (self-hostable)
- Sources:
  - [GitHub: amedinat/LLMeter](https://github.com/amedinat/LLMeter)
  - [llmeter.app](https://llmeter.app)

## DOCUMENTATION

### Overview

LLMeter is an open-source AI cost dashboard that pulls usage data directly from provider billing APIs for OpenAI, Anthropic, DeepSeek, OpenRouter, Google AI, and Mistral using read-only API keys. It does not use a proxy, SDK, or code changes. It is aimed at developers and small teams watching API spend, not CLI-based subscription users.

### Claimed features

- Multi-provider support: OpenAI, Anthropic, Google AI, DeepSeek, OpenRouter, and Mistral.
- Reads from provider billing APIs using read-only keys.
- Budget alerts before surprise invoices.
- Real-time cost breakdowns by model.
- AES-256-GCM encryption for stored API keys.
- Self-hostable deployment.

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

- Does LLMeter plan to add local-file or CLI session data sources?
- Is there a roadmap for Claude Code / Codex subscription quota tracking?
