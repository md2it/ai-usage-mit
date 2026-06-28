# ccusage

## META

- Date researched: 2026-06-28
- URL: https://ccusage.com
- GitHub: https://github.com/ryoppippi/ccusage
- Relevance: High
- Pricing: Free (MIT open-source)
- License: MIT
- GitHub: 14,100 stars, 549 forks (as of 2026-06-28)

## OVERVIEW

ccusage is a zero-install CLI tool (`npx ccusage`) that reads local JSONL telemetry files produced by Claude Code, Codex CLI, Gemini CLI, and 12+ other coding agent CLIs, then generates daily, weekly, monthly, and per-session usage reports. No proxying, no traffic interception — pure local file analysis. Free and open-source. It also ships a companion web dashboard at ccusage.com for aggregated stats.

## FEATURES

- 15+ supported tools: Claude Code, Codex, Gemini CLI, GitHub Copilot CLI, Goose, Amp, OpenCode, and more
- Daily / weekly / monthly / per-session breakdowns
- Custom pricing overrides per model
- Claude Pro/Max instance grouping by project
- Timezone-aware date grouping
- Ultra-small bundle — runs via `npx` / `bunx`, zero install
- Web dashboard at ccusage.com with aggregated stats across multiple machines

## PRICING / BUSINESS MODEL

Fully free and open-source (MIT). No paid tier, no SaaS subscription. Revenue model unknown; likely a passion project or pre-commercial.

## COMPARISON WITH AI-USAGE-MIT

| Dimension | ccusage | ai-usage-mit |
|---|---|---|
| **Core value** | Historical usage reports from local JSONL files | Subscription quota visibility, burn-rate, reset countdown |
| **Multi-tool support** | Yes — 15+ tools | — |
| **Data source** | Local JSONL files (post-session) | Local session data / telemetry |
| **Real-time** | No — reads files after sessions complete | — |
| **Quota / limit tracking** | No — no subscription progress bar or reset countdown | Core feature |
| **Burn-rate prediction** | No — "will I hit my limit?" warnings absent | — |
| **Web UI** | ccusage.com (external) | — |
| **License** | MIT | — |

### Threat level

**High.** Most direct open-source competitor: same local-file data strategy, same multi-tool scope (Claude Code + Codex + Gemini CLI), already at 14k stars. The critical gap ccusage leaves: subscription quota visualization (progress toward limit, reset countdown) and real-time burn-rate prediction — which is ai-usage-mit's core differentiator.

## OPEN QUESTIONS

- Does ccusage.com aggregate data server-side or is it a static frontend?
- Is there a maintainer roadmap — are quota/subscription features planned?
- How does it handle the 30-day JSONL cleanup Claude Code performs?

## SOURCES

- [GitHub: ryoppippi/ccusage](https://github.com/ryoppippi/ccusage)
- [ccusage.com](https://ccusage.com)
