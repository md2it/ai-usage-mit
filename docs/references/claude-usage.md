# claude-usage (phuryn)

## CONCLUSION

### After documentation

- claude-usage is a local web dashboard for Claude Code usage history.
- It is Claude Code only.
- It is free and open-source under MIT.
- It is also available as a VS Code extension and via Homebrew.

### After hands-on

- A controlled Claude Code-style JSONL transcript was scanned into SQLite.
- CLI reports for today, week, and stats worked and showed model split, token categories, session count, project name, and estimated API cost.
- The local dashboard served successfully when local port binding was allowed.
- Dashboard API aggregates and UI controls were confirmed from local behavior and source.
- Additional provider check on 2026-06-28 scanned real local Codex JSONL files from `~/.codex`, but added 0 turns, saw 0 sessions, and reported `$0.0000` cost.
- Control run against real `~/.claude` in the same setup added 15,556 turns, saw 167 sessions, and showed Claude cost for today.
- Chart rendering depends on CDN access unless Chart.js is cached or vendored.
- A separate Pro/Max subscription quota progress bar was not found in the tested dashboard code or rendered HTML.

### Comparison to ai-limits

- Core value: claude-usage provides a local web dashboard for Claude Code usage history; ai-limits targets subscription quota visibility across multiple tools.
- Tool support: claude-usage is Claude Code only.
- UI: claude-usage has a localhost web dashboard.
- VS Code integration: available.
- Real-time behavior: partial, with 30-second auto-refresh.
- Burn-rate prediction: not observed.
- Multi-provider view: not present.
- Threat level: Medium. It uses the same local-file approach and has useful web dashboard UX, but its scope is Claude-only and no separate burn-rate or subscription limit warning was observed.

### What we can learn

- A localhost dashboard is a useful UX step beyond terminal-only reporting.
- Date and model filters are baseline controls for usage review.
- CSV export from cost/session tables is useful for users who reconcile usage externally.
- CDN dependencies should be considered carefully for local-first products.
- Its main source is `~/.claude/projects/**/*.jsonl`, but it also checks Xcode's Claude agent path: `~/Library/Developer/Xcode/CodingAssistant/ClaudeAgentConfig/projects`.
- Scanning into a local SQLite database (`~/.claude/usage.db` by default) gives fast dashboard reads and incremental rescans without treating SQLite as the original source of truth.
- Claude discovery should include client-specific transcript roots, not only the standard Claude Code directory.

## META

- Date researched: 2026-06-28
- URL: https://github.com/phuryn/claude-usage
- Relevance: Medium
- Pricing: Fully free and open-source. No paid tier.
- License: MIT
- GitHub: ~1,600 stars (as of 2026-06-28)
- Hands-on checked: 2026-06-28, repo commit `c60aaa4a756d4c6833c7b2f7a7b9b9a292099e69`, runtime version `1.5.0`
- Sources:
  - [GitHub: phuryn/claude-usage](https://github.com/phuryn/claude-usage)

## DOCUMENTATION

### Overview

claude-usage is a local web dashboard for tracking Claude Code token usage, costs, and session history. It scans Claude Code-style JSONL transcripts into SQLite, produces terminal summaries, and serves a single-page dashboard with charts, filters, sortable/exportable tables, and auto-refresh for ranges that include today.

### Claimed features

- Web dashboard on localhost with no cloud, proxy, or account required.
- Reads Claude Code JSONL transcripts into a local SQLite database.
- CLI reports: `scan`, `today`, `week`, `stats`, and `dashboard`.
- Auto-refresh every 30 seconds.
- Model filtering and date-range selection.
- Chart.js usage charts.
- Sortable cost/session tables and CSV export.
- VS Code extension.
- Homebrew installable.

## HANDS-ON CHECK

### Overview

Test setup: cloned `https://github.com/phuryn/claude-usage.git` to `/private/tmp/claude-usage-phuryn`; created a controlled Claude Code-style JSONL transcript under `/private/tmp/claude-usage-handson/projects/test-project/session-001.jsonl`; used `CLAUDE_USAGE_DB=/private/tmp/claude-usage-handson/usage.db` so the test did not write to the real `~/.claude/usage.db`.

### Installing

The tested path used the cloned repository directly with `python3 cli.py`. It worked without pip install or virtualenv. VS Code extension and Homebrew installation were reviewed from docs/source only and were not installed.

### Data access

The tool scanned Claude Code-style JSONL transcripts from the provided `--projects-dir` into a local SQLite database specified by `CLAUDE_USAGE_DB`.

Additional provider check on 2026-06-28:

- Real local Codex data exists at `/Users/alekseyterekhov/.codex`
- `CLAUDE_USAGE_DB=... python3 cli.py scan --projects-dir /Users/alekseyterekhov/.codex` enumerated many Codex JSONL files, including `archived_sessions`, `sessions`, `history.jsonl`, and `session_index.jsonl`
- Despite enumerating those files, the scan summary was `Turns added: 0` and `Sessions seen: 0`
- `python3 cli.py stats` on the Codex database returned `Total sessions: 0`, `Total turns: 0`, and `Est. total cost: $0.0000`
- Control run against `/Users/alekseyterekhov/.claude` returned `Turns added: 15556`, `Sessions seen: 167`, and today's Claude model cost

### Verified behavior

- `python3 cli.py scan --projects-dir /private/tmp/claude-usage-handson/projects` added 2 turns / 1 session to SQLite.
- `today`, `week`, and `stats` reports showed model split, input/output tokens, cache read, cache creation, session count, project name, and estimated API cost.
- `python3 cli.py dashboard --projects-dir ... --host 127.0.0.1 --port 18080 --no-browser` served the dashboard successfully when local port binding was allowed.
- `/api/data` returned expected aggregates: `all_models`, `daily_by_model`, `hourly_by_model`, `sessions_all`, `subagent_by_type`, and `top_dispatches`.
- UI/code exposed model multi-select, date-range dropdown, Rescan button, jump navigation, collapsible sections, sortable cost tables, CSV export, and 30-second auto-refresh for ranges including today.

### Problems

- Dashboard HTML loads Chart.js from `https://cdn.jsdelivr.net/npm/chart.js@4.4.0/dist/chart.umd.min.js`; server and data are local, but chart rendering depends on CDN access unless cached or changed.
- No separate Pro/Max subscriber quota progress bar was found in current dashboard code or rendered HTML. The README says "Pro and Max subscribers get a progress bar", but the observed app exposes token/cost dashboarding rather than a visible subscription quota progress bar.
- Multi-provider support was not observed in behavior. The scanner can walk a Codex directory, but it does not extract Codex turns, token totals, cost, limits, or reset information from the tested Codex JSONL data.
- VS Code extension was not installed in VS Code during this hands-on pass.
- Homebrew formula was inspected but not installed.

## OPEN QUESTIONS

- Is multi-tool support planned?
- Does the README's Pro/Max progress-bar claim refer to an older UI, an untested data condition, or a planned feature?
