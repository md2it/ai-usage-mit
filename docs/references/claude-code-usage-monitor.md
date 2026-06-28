# Claude-Code-Usage-Monitor

## CONCLUSION

### After documentation

- Claude-Code-Usage-Monitor is a real-time terminal monitor for Claude Code usage.
- It focuses on live burn-rate tracking, session limit prediction, and terminal status visibility.
- It targets individual Claude Code Pro and Max subscribers.
- It is Claude Code only.

### After hands-on

- `claude-monitor==4.0.0` was installed in a local virtualenv and CLI aliases/options were confirmed.
- The tool parsed local Claude Code JSONL data and identified `source.kind=claude_code_jsonl`.
- Live Rich terminal UI was confirmed with cost, token, and message progress bars, reset time, model distribution, burn rate, cost rate, and predictions.
- Burn rate and prediction were confirmed on real local data.
- Local history fields were returned by the CLI.
- Cost usage and cost-rate estimates were observed, but a separate alert workflow was not triggered.
- Long-term retention behavior was not verified.

### Comparison to ai-usage-mit

- Core value: Claude-Code-Usage-Monitor provides real-time burn-rate and session limit prediction for Claude Code; ai-usage-mit targets subscription quota visibility across tools.
- Tool support: Claude-Code-Usage-Monitor is Claude Code only.
- Real-time UI: confirmed terminal live UI.
- Burn-rate and prediction: confirmed.
- Web UI and VS Code UI: not observed.
- Multi-provider view: not present.
- Threat level: High. It overlaps strongly with the Claude Code segment: local-file approach, quota/burn-rate angle, and high organic demand. ai-usage-mit can differentiate through multi-tool support and a polished UI beyond terminal output.

### What we can learn

- Live burn-rate and time-to-limit forecasts are high-value signals.
- Terminal statusline support is useful for users already working in CLI contexts.
- Local history that survives upstream cleanup is important, but needs clear verification.
- Multi-tool coverage remains a likely differentiator against Claude-only monitors.

## META

- Date researched: 2026-06-28
- URL: https://github.com/Maciek-roboblog/Claude-Code-Usage-Monitor
- Relevance: High
- Pricing: Fully free and open-source. No paid tier.
- License: Unknown
- GitHub: 8,300 stars (as of 2026-06-28)
- Hands-on tested: 2026-06-28, package `claude-monitor==4.0.0`
- Sources:
  - [GitHub: Maciek-roboblog/Claude-Code-Usage-Monitor](https://github.com/Maciek-roboblog/Claude-Code-Usage-Monitor)

## DOCUMENTATION

### Overview

Claude-Code-Usage-Monitor is a real-time terminal monitor for Claude Code usage. Written in Python, it uses the Rich library to render a live-updating terminal UI with color-coded progress bars, burn-rate tracking, and predictive warnings about when the current session's limit will be hit. It is targeted at individual Claude Code Pro and Max subscribers and does not cover Codex, Gemini CLI, or other tools.

### Claimed features

- Real-time monitoring with live-updating Rich terminal UI.
- Burn rate analysis for token consumption velocity.
- Session limit prediction with expected limit-hit time.
- Budget alerts and cost projections.
- Progress bars for usage windows.
- Versioned local history intended to survive Claude's 30-day JSONL cleanup.
- 100+ test cases and active maintenance.

## HANDS-ON CHECK

### Overview

Test setup: installed PyPI package `claude-monitor==4.0.0` into a local virtualenv at `.runtime/claude-monitor-venv` and ran the CLI as `.runtime/claude-monitor-venv/bin/claude-monitor`. To avoid writes outside the repo during testing, `HOME` was pointed to `.runtime/ccm-home`.

### Installing

The tested install used a local virtualenv and the PyPI package `claude-monitor==4.0.0`. Installed commands included `claude-monitor`, `claude-code-monitor`, `ccmonitor`, `cmonitor`, and `ccm`.

### Data access

With `--data-paths /Users/alekseyterekhov/.claude`, the tool reported `source.kind=claude_code_jsonl` and parsed local Claude Code data. With no Claude JSONL in the provided `.claude` directory, the tool returned `status.label=no_active_session`, zero tokens, and no forecast.

### Verified behavior

- `--help` exposed plans (`pro`, `max5`, `max20`, `team`, `custom`), views (`realtime`, `daily`, `monthly`, `session`, `entries`, `sessions`, `burn-rate`), one-shot output (`rich`, `json`, `text`, `csv`), `--statusline`, `--write-state`, experimental `--api`, and opt-in `--warehouse`.
- Live mode rendered a Rich terminal screen with cost, token, and message progress bars, time-to-reset, model distribution, burn rate, cost rate, and predictions.
- A one-shot JSON run returned an active session, `burn_rate_tokens_per_minute`, `burn_rate_cost_per_hour`, `pace.label`, and `forecast.predicted_tokens_exhausted_at`.
- The same run returned `local_history.total_tokens` and `local_history.total_cost_usd`.
- With real `~/.claude` data, the tested snapshot returned `status.label=ok`, active 5-hour session data, token/cost/message usage, model distribution, burn rate, reset time, and forecast.

### Problems

- Budget alerts / cost projections were only partially confirmed. UI and JSON showed cost usage, cost rate, and cost-based session limit estimates; no separate alert workflow was triggered during the short test.
- Versioned local history surviving Claude cleanup was not directly verified. The CLI exposes `--warehouse` and can create a warehouse file, but long-term retention behavior was not tested.
- Multi-provider support was not present in tested CLI surface; the only model filter option seen was `--filter-models {all,anthropic}`.
- Web or VS Code UI was not seen.
- The CLI writes under `~/.claude-monitor` by default, even for `--once`. In the sandboxed first run it failed with `PermissionError` when it could not create `/Users/alekseyterekhov/.claude-monitor`.

## OPEN QUESTIONS

- Does it plan to add Codex or Gemini CLI support?
- Is there a roadmap for a web/desktop UI?
- How reliable is the opt-in warehouse retention across Claude cleanup over weeks/months?
