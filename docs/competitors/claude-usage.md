# claude-usage (phuryn)

## META

- Date researched: 2026-06-28
- URL: https://github.com/phuryn/claude-usage
- Relevance: Medium
- Pricing: Free (open-source)
- License: Unknown
- GitHub: ~1,600 stars (as of 2026-06-28)

## OVERVIEW

claude-usage is a local web dashboard for tracking Claude Code token usage, costs, and session history. It serves a single-page app at localhost:8080 with Chart.js charts, auto-refreshes every 30 seconds, supports model filtering and date-range selection, and shows a usage progress bar for Pro/Max subscribers. Also available as a VS Code extension and installable via Homebrew. Claude Code only. Free.

## FEATURES

- Web dashboard on localhost — no cloud, no proxy, no account required
- Usage progress bar for Pro/Max subscriber quota
- Auto-refresh every 30 seconds
- Model filtering and date-range selection
- Chart.js usage charts
- VS Code extension
- Homebrew installable (no pip/venv required)

## PRICING / BUSINESS MODEL

Fully free and open-source. No paid tier.

## COMPARISON WITH AI-USAGE-MIT

| Dimension | claude-usage (phuryn) | ai-usage-mit |
|---|---|---|
| **Core value** | Local web dashboard for Claude Code usage history | Subscription quota visibility across multiple tools |
| **Tool support** | Claude Code only | Multi-tool (Claude Code, Codex, Gemini CLI, …) |
| **UI** | Web dashboard (localhost) | — |
| **VS Code integration** | Yes — extension available | — |
| **Real-time** | Partial — 30s auto-refresh | — |
| **Burn-rate prediction** | No — no "will I hit my limit?" warnings | — |
| **Multi-provider view** | No | — |

### Threat level

**Medium.** Same local-file approach, same Claude Code audience, web dashboard UX is a legitimate UI strength. Loses on scope (Claude-only) and lacks burn-rate/prediction features. Less actively maintained than ccusage or Claude-Code-Usage-Monitor. ai-usage-mit beats it by covering multi-tool scope and trend analysis.

## OPEN QUESTIONS

- Is the VS Code extension actively maintained?
- Is multi-tool support planned?

## SOURCES

- [GitHub: phuryn/claude-usage](https://github.com/phuryn/claude-usage)
