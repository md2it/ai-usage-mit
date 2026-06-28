# LLMeter

## CONCLUSION

### After documentation

- LLMeter is an open-source AI cost dashboard for provider billing APIs.
- It targets API spend visibility, not CLI subscription quota tracking.
- It exposes API usage, real cost, historical trends/retention, budget alerts, anomaly alerts, and exports; these are API spend controls, not Claude Code / Codex / Gemini CLI subscription quotas.
- It avoids proxying for providers with billing APIs, and uses SDK wrappers or ingestion for providers/attribution cases where billing APIs are insufficient.
- It explicitly cannot block or throttle API calls in real time.

### After hands-on

Not checked yet.

### Comparison to ai-usage-mit

- Core value: LLMeter is a multi-provider API billing cost dashboard; ai-usage-mit focuses on subscription quota visibility for CLI-based coding tools.
- Data source: LLMeter uses provider billing APIs; ai-usage-mit is expected to use local CLI session data or telemetry.
- Target user: LLMeter targets developers with direct API spend; ai-usage-mit targets users of Claude Code, Codex, Gemini CLI, and similar subscriptions.
- CLI subscription awareness: LLMeter does not cover Claude Code, Codex, or Gemini CLI subscription quotas.
- Limits/credits/reset: docs show configurable daily/monthly spend alerts and provider-native soft limits as context, but no real subscription quota cap, remaining credits balance, or reset countdown for CLI tools.
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
- Hosted docs conflict slightly on Google AI: homepage FAQ says "Google AI (Gemini) coming soon," while README lists Google AI via SDK wrapper.
