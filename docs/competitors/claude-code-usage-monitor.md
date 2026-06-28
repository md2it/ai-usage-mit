# Claude-Code-Usage-Monitor

## META

- Date researched: 2026-06-28
- URL: https://github.com/Maciek-roboblog/Claude-Code-Usage-Monitor
- Relevance: High
- Pricing: Free (open-source)
- License: Unknown
- GitHub: 8,300 stars (as of 2026-06-28)

## OVERVIEW

Claude-Code-Usage-Monitor is a real-time terminal monitor for Claude Code usage. Written in Python, it uses the Rich library to render a live-updating terminal UI with color-coded progress bars, burn-rate tracking, and predictive warnings about when the current session's limit will be hit. Targeted at individual Claude Code Pro and Max subscribers. Free. Claude Code only — no Codex, Gemini CLI, or other tools.

## FEATURES

- Real-time monitoring with live-updating terminal UI (Rich)
- Burn rate analysis: tracks token consumption velocity
- Session limit prediction: warns when the limit will be hit and by what time
- Budget alerts and cost projections
- Progress bars for usage windows
- Versioned local history (survives Claude's 30-day JSONL cleanup)
- 100+ test cases, actively maintained

## PRICING / BUSINESS MODEL

Fully free and open-source. No paid tier.

## COMPARISON WITH AI-USAGE-MIT

| Dimension | Claude-Code-Usage-Monitor | ai-usage-mit |
|---|---|---|
| **Core value** | Real-time burn-rate and session limit prediction for Claude Code | Subscription quota visibility across tools |
| **Tool support** | Claude Code only | Multi-tool (Claude Code, Codex, Gemini CLI, …) |
| **Real-time** | Yes — live-updating terminal UI | — |
| **Burn-rate / prediction** | Yes — "will I hit my limit?" with time estimate | Core feature |
| **Web UI** | No — terminal only | — |
| **VS Code integration** | No | — |
| **Multi-provider view** | No | — |

### Threat level

**High.** Almost identical problem space to ai-usage-mit for the Claude Code user segment: same local-file approach, same quota/burn-rate angle, strong organic demand (8k+ stars). Key differentiator ai-usage-mit can claim: multi-tool support (one view for Codex + Gemini CLI + others) and a polished UI/UX beyond a terminal script.

## OPEN QUESTIONS

- Does it plan to add Codex or Gemini CLI support?
- Is there a roadmap for a web/desktop UI?

## SOURCES

- [GitHub: Maciek-roboblog/Claude-Code-Usage-Monitor](https://github.com/Maciek-roboblog/Claude-Code-Usage-Monitor)
