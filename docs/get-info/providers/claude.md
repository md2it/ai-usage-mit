# Claude

## Current Status

PoC uses two Claude sources and one live-limit method:

- `claude_cli_usage`: launches `claude --no-chrome`, sends `/usage`, parses TUI lines.
- `claude_local_usage`: scans local transcript JSONL files and aggregates token usage history.
- `claude_statusline_rate_limits`: reads Claude Code statusline stdin payload and extracts live `rate_limits` when available.

---

## Provider Method: `claude_cli_usage`

Minimum commands:

- check that the CLI is installed: `command -v claude`
- check CLI version: `claude --version`
- official site: https://www.anthropic.com/claude-code
- CLI documentation: https://code.claude.com/docs/en/setup

Verified PoC details:

- the standard `claude` command is run with the `--no-chrome` flag to avoid opening the additional Chrome integration dialog
- `/usage` is used to retrieve limits
- `/status` opens the Status tab by default without limits
- the PoC waits for the prompt to be ready based on the bottom line `for shortcuts`
- `/usage` is sent as regular input without bracketed paste
- user-facing output shows the matched lines `Current session`, `Current week`, `Total cost`, and token usage
- structured limits map `Current session` to a 5-hour window (`window_minutes = 300`) and `Current week` to a 7-day window (`window_minutes = 10080`)
- the parser accounts for some lines arriving via bare carriage return, so cleaned/compacted output is split on `\n` and `\r`

---

## Provider Method: `claude_local_usage`

Minimal sources:

- `~/.config/claude/projects`
- `~/.claude/projects`
- `~/Library/Developer/Xcode/CodingAssistant/ClaudeAgentConfig/projects`

What is extracted:

- `assistant` records with non-zero `message.usage`
- deduplicated turns by `message.id` (latest record wins in file)
- scope summary: files, sessions, turns
- token totals: input/output/cache-read/cache-write/total
- top model and latest activity timestamp

Behavior:

- if no local roots are present, returns `local transcript roots were not found`
- if roots exist but no token usage is found, returns `no token usage found`
- local transcripts provide usage history; official remaining limit/reset may be unavailable

### Local limit reconstruction notes

Research date: 2026-06-30.

Confirmed from local code and hands-on checks:

- Claude Code local transcript JSONL files include assistant usage records with:
  - `input_tokens`
  - `output_tokens`
  - `cache_read_input_tokens`
  - `cache_creation_input_tokens`
- `claude-code-usage-monitor` reconstructs the displayed 5-hour token usage from local JSONL as `input_tokens + output_tokens`.
- `claude-code-usage-monitor` does not appear to retrieve the 5-hour token denominator from an official Claude API in the tested path. It uses local plan constants.
- In `claude-code-usage-monitor==4.0.0`, the plan constants include:
  - `max5`: `88_000` tokens
  - `max20`: `220_000` tokens
- `cache_read_input_tokens` and `cache_creation_input_tokens` are useful for history/cost reporting, but they are not the numerator used by `claude-code-usage-monitor` for the 5-hour limit bar.
- `claude-code-usage-monitor --plan max5 --once --data-paths ~/.claude` showed the same kind of 5-hour live estimate that ai-limits needs for `--claude-local`.

Current ai-limits implementation state:

- `--claude-local` reconstructs an active 5-hour window from local transcripts.
- The current implementation uses `input_tokens + output_tokens` and a local `88_000` token denominator for the 5-hour window.
- This is a local estimate, not an official Claude limit signal.
- Current reset calculation uses a local reconstructed window and is less reliable than the token numerator.
- The reset estimate must not be presented as official when no live `rate_limits` snapshot is available.

Reset reconstruction findings from 2026-07-02:

- The reliable formula is still `window_start + 5h`; the weak point is determining `window_start`.
- `round_down(first_turn_timestamp_to_hour) + 5h` is too coarse and caused observed reset drift.
- `first transcript turn after a >=5h inactivity gap + 5h` is better than hourly rounding but still incomplete.
- In observed data, this gap-only heuristic still missed the trusted reset because the true server window boundary can fall inside an active transcript period.
- The best local improvement is to use the latest known server reset as an anchor when one has been captured from `rate_limits`, `/usage`, `rate_limit_event`, or a 429 payload.
- Without a server anchor, `claude_local_usage` remains an estimate for reset timing.

Important 5-hour observation from 2026-06-30:

- The `88_000` denominator matched earlier observed behavior and is also commonly reported by community research, but it did not match one later high-usage session.
- During that later session, local transcript usage for the active 5-hour window was about `99,630` input+output tokens.
- Claude CLI and other Claude interfaces still showed the account as inside the 5-hour window, with about `8%` remaining.
- The same mismatch was observed in `claude-code-usage-monitor`, which reported over-100% usage when using the `88_000` denominator.
- Therefore, `88_000` should be treated as an approximate/community-derived Max5 5-hour denominator, not a universally reliable live limit.
- Possible explanations include dynamic account limits, burst tolerance, model/session-specific behavior, delayed enforcement, or an incomplete local reconstruction model. None are confirmed.

Important 7-day observation from 2026-06-30:

- Claude CLI and other Claude interfaces showed the weekly limit at approximately:
  - `80%` used / `20%` left
  - later `83%` used / `17%` left
  - nearest observed weekly reset: `2026-06-30 13:00 UTC+3`
- Local transcript reconstruction for the assumed 7-day window ending at that reset produced:
  - at `80%`: about `1,045,761` input+output tokens
  - at `83%`: about `1,085,815` input+output tokens
- These imply a stable weekly denominator around:
  - `1,045,761 / 0.80 ~= 1.307M`
  - `1,085,815 / 0.83 ~= 1.308M`
- This is a strong working hypothesis that the 7-day local numerator is also `input_tokens + output_tokens`.
- The exact 7-day denominator, exact window start, and exact rounding method are not confirmed.
- The reset timestamp came from Claude UI/CLI observation, not from local transcript JSONL.

Future weekly reset checkpoints to watch:

- `2026-06-30 13:00 UTC+3` — nearest observed reset during this investigation.
- If the weekly window is exactly 7 days and the reset schedule remains stable, the next candidate reset is `2026-07-07 13:00 UTC+3`.
- This recurrence is a hypothesis for future validation, not a confirmed provider rule.

Do not treat as confirmed yet:

- exact Max5 5-hour denominator
- exact Max5 7-day denominator
- whether denominators are static per plan
- whether Claude uses the same denominator across accounts, models, regions, and account states
- whether the weekly window is always exactly `reset - 7 days`
- whether the CLI/UI percentage is rounded, floored, or otherwise bucketed

---

## Provider Method: `claude_statusline_rate_limits`

Method details: [../methods/statusline.md](../methods/statusline.md).

Minimal source:

- Claude Code statusline stdin payload
- no TUI parsing and no transcript reconstruction for current limits

How to get data:

1. configure a Claude Code statusline command in `~/.claude/settings.json` or `~/.config/claude/settings.json`
2. Claude Code runs the configured command when it refreshes the statusline
3. Claude Code passes a JSON payload to the command through stdin
4. the command saves the latest valid payload to a local ai-limits cache file
5. ai-limits reads the cache file and parses `rate_limits`
6. ai-limits normalizes live fields for current windows (5h/7d), used progress, and reset time

Behavior:

- when statusline payload includes `rate_limits`, this method provides an official live signal for current Claude limits
- when statusline context is unavailable or payload has no `rate_limits`, method returns unavailable/unknown for live limits
- this method is for current live limits, not full historical usage aggregation
- this method primarily covers Claude Code CLI; Claude Desktop and browser-extension flows are not confirmed to run the same statusline command

---

## Limitations

- for `claude_cli_usage`, full output remains a TUI stream and depends on current CLI text/layout
- for `claude_cli_usage`, request/parse can take noticeable time
- for `claude_statusline_rate_limits`, data is available only after a properly configured Claude Code statusline command receives a payload
- for `claude_statusline_rate_limits`, unavailable statusline context means live limits are unavailable even if transcript history exists
- for `claude_local_usage`, reset remains an estimate unless a server reset anchor is available

---

## Other Options

| Option | Status | Comment |
|---|---|---|
| Official API | Not investigated | May apply to API accounts, but not necessarily to Claude Code subscription limits |
| Local transcript JSONL (`claude_local_usage`) | Implemented in PoC | Scans local transcript roots and aggregates token usage history by assistant turns; official remaining limit/reset may be unavailable |
| Claude Code statusline `rate_limits` | Live-limit method | Statusline command receives JSON via stdin from Claude Code and can provide an official live signal for 5h/7d limits; requires statusline configuration and cache capture |
| Local SQLite/cache | Auxiliary layer | e.g. `~/.claude/usage.db` from `claude-usage`: convenient for dashboard and incremental scanning, but this is derived data, not a primary source |
| Frontend/dashboard API | Research-only | Possible only with a clear and safe way to handle cookie/session tokens |
| Traffic observation | Research-only | Not to be considered as a product mechanism |
