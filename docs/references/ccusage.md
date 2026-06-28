# ccusage

## CONCLUSION

### After documentation

- ccusage is a zero-install CLI tool for local usage reports from AI coding agent telemetry files.
- It supports Claude Code, Codex CLI, Gemini CLI, and many other coding agent CLIs.
- It is free and open-source under MIT.
- It also has a companion web dashboard at ccusage.com.

### After hands-on

- `npx ccusage` installed and ran `ccusage@20.0.14` locally on 2026-06-28.
- Local Claude and Codex usage were detected in offline JSON mode.
- Daily, weekly, monthly, session, and active Claude block reports worked on local data.
- Active Claude billing blocks exposed burn rate, remaining minutes, projected total tokens, and projected total cost.
- Subscription-plan quota progress or reset countdown for Codex/Gemini-style subscriptions was not observed.
- The web dashboard at ccusage.com was not tested hands-on.

### Comparison to ai-usage-mit

- Core value: ccusage provides historical usage reports from local JSONL files; ai-usage-mit focuses on subscription quota visibility, burn-rate, and reset countdown.
- Multi-tool support: ccusage already supports 15+ tools, including Claude Code, Codex, and Gemini CLI.
- Data source: ccusage reads local JSONL files after usage.
- Real-time behavior: mostly post-session/local-file reporting, with Claude statusline and active block features.
- Quota/limit tracking: partial. Claude session-block token limits and projections were observed, but no subscription progress bar or reset countdown was observed.
- Threat level: High. ccusage is a direct open-source analog with local-file strategy, multi-tool scope, and strong adoption. The main remaining gap for ai-usage-mit is broader subscription quota visualization across supported tools.

### What we can learn

- Zero-install execution through `npx` lowers adoption friction.
- Offline JSON output is important for users who want local-only workflows.
- Agent-specific namespaces help keep multi-tool reporting understandable.
- Active-block projections are useful, but subscription-specific reset and quota views remain a differentiator to evaluate.

## META

- Date researched: 2026-06-28
- URL: https://ccusage.com
- GitHub: https://github.com/ryoppippi/ccusage
- Relevance: High
- Pricing: Fully free and open-source. No paid tier or SaaS subscription documented.
- License: MIT
- GitHub: 14,100 stars, 549 forks (as of 2026-06-28)
- Hands-on tested: 2026-06-28 with `npx ccusage`; npm installed and ran `ccusage@20.0.14`.
- Sources:
  - [GitHub: ryoppippi/ccusage](https://github.com/ryoppippi/ccusage)
  - [ccusage.com](https://ccusage.com)

## DOCUMENTATION

### Overview

ccusage is a zero-install CLI tool (`npx ccusage`) that reads local JSONL telemetry files produced by Claude Code, Codex CLI, Gemini CLI, and 12+ other coding agent CLIs, then generates daily, weekly, monthly, and per-session usage reports. It does not proxy or intercept traffic. It also ships a companion web dashboard at ccusage.com for aggregated stats.

### Claimed features

- 15+ supported tools: Claude Code, Codex, Gemini CLI, GitHub Copilot CLI, Goose, Amp, OpenCode, and more.
- Daily, weekly, monthly, and per-session breakdowns.
- Session billing blocks with active-block projection and optional token-limit warnings.
- Custom pricing overrides per model.
- Claude Pro/Max instance grouping by project.
- Timezone-aware date grouping.
- Small bundle that runs via `npx` / `bunx`.
- Web dashboard at ccusage.com with aggregated stats across multiple machines.

## HANDS-ON CHECK

### Overview

Tested locally on 2026-06-28 with `npx ccusage`; npm installed and ran `ccusage@20.0.14`. The purpose was to verify local multi-agent reporting and active usage projection behavior.

### Installing

The tested path was zero-install execution through `npx ccusage`. No persistent install workflow was documented in this check.

### Data access

`npx ccusage daily --json --offline` detected local Claude and Codex usage from local telemetry files and returned daily rows with `metadata.agents`, model breakdowns, input/output/cache tokens, total tokens, and estimated cost.

### Verified behavior

- `npx ccusage --help` exposed aggregate commands: `daily`, `weekly`, `monthly`, `session`, `blocks`, and `statusline`.
- Help also exposed agent-specific namespaces for Claude, Codex, OpenCode, Amp, Droid, Codebuff, Hermes, pi-agent, Goose, Kilo, GitHub Copilot CLI, Gemini, Kimi, Qwen, and OpenClaw.
- `npx ccusage codex daily --since 2026-06-28 --json --offline` returned Codex-only usage for 2026-06-28, including reasoning output tokens in metadata.
- `npx ccusage monthly --since 2026-06-01 --json --offline`, `weekly`, and `session` worked on local data and produced grouped reports.
- `npx ccusage blocks --active --json --offline` returned an active Claude session billing block with start/end time, burn rate, remaining minutes, projected total tokens, and projected total cost.
- Help output confirmed timezone filtering, JSON output, date filtering, offline pricing, no-cost mode, compact mode, and config file support.

### Problems

- Subscription-plan quota progress or reset countdown for Codex/Gemini-style subscriptions was not observed in tested commands.
- The observed `blocks` feature is Claude session-block oriented and is not a general subscription quota dashboard.
- The companion web dashboard at ccusage.com was not tested hands-on, so server-side aggregation remains unverified.
- Custom pricing overrides were visible only as config support in help; no custom pricing file was created or tested.

## OPEN QUESTIONS

- Does ccusage.com aggregate data server-side or is it a static frontend? Not tested hands-on.
- Is there a maintainer roadmap, and are quota/subscription features planned?
- How does it handle the 30-day JSONL cleanup Claude Code performs?
