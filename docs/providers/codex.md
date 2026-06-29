# Codex

## Current status

PoC uses two Codex sources:

- `codex_cli_usage`: launches `codex`, sends `/status`, parses TUI limit lines.
- `codex_local_usage`: scans local JSONL history in `${CODEX_HOME:-~/.codex}`, aggregates token usage, and reads local rate-limit snapshots.

---

## Provider Method: `codex_cli_usage`

Minimum commands:

- verify CLI availability: `command -v codex`
- verify CLI version: `codex --version`
- official website: https://openai.com/codex
- CLI documentation: https://developers.openai.com/codex/cli

Verified PoC details:

- launches the standard `codex` command without a custom path to the CLI
- Codex CLI refuses to launch the interactive TUI if `stdin`/`stderr` are not TTYs
- for PoC, the system `expect` command is used as a minimal PTY adapter
- runtime sets `TERM=xterm-256color`, `COLUMNS=120`, `LINES=40` and runs `stty cols 120 rows 40`
- PoC sends `/status` via bracketed paste
- the first `/status` call sometimes triggers a limit refresh
- a second `/status` call returns the actual breakdown
- the parser waits for response indicators: startup screen, `refresh requested`, limit lines, or `Credits`
- user-facing output shows only the found summary: `5h limit`, `Weekly limit`, and `Credits`

---

## Provider Method: `codex_local_usage`

Minimal source:

- root: `${CODEX_HOME:-~/.codex}`
- scanned directories: `sessions/`, `archived_sessions/`
- scanned files: `**/*.jsonl`

What is extracted:

- events with `"type":"token_count"` and `"last_token_usage"`
- totals: input, cached input, output, reasoning output, total
- latest activity timestamp (ISO 8601 UTC from the latest `token_count` event with `rate_limits`)
- `rate_limits` snapshot when present: `primary.used_percent`, `primary.window_minutes`, `primary.resets_at`, `secondary.used_percent`, `secondary.window_minutes`, `secondary.resets_at`, `credits`, `plan_type`

How to get these fields from local files:

1. read `${CODEX_HOME:-~/.codex}/sessions/**/*.jsonl` and `${CODEX_HOME:-~/.codex}/archived_sessions/**/*.jsonl`
2. keep only records where `type = "event_msg"` and `payload.type = "token_count"`
3. for usage, aggregate `payload.info.last_token_usage.*`
4. for limits/reset, read `payload.rate_limits.*` from the latest timestamped event that includes `rate_limits`
5. show `Latest activity` and `resets_at` as ISO 8601 UTC (`YYYY-MM-DDTHH:MM:SSZ`)

Behavior:

- if root is missing, returns `not found`
- if no token events are found, returns `token events: not found`
- local Codex JSONL can provide current local snapshot for limit percent and reset time (5h/weekly windows when present)
- local Codex JSONL usually does not provide absolute quota size (`used_tokens`/`max_tokens`), only percent and reset window

---

## Limitations

- full output remains a TUI stream and may contain terminal control sequences
- the approach depends on the current CLI behavior and TUI text
- CLI requests can take a noticeable amount of time
- needs verification of whether such requests consume user limits

---

## Other options

| Option | Status | Comment |
|---|---|---|
| Official API | Not investigated | Requires separate verification of usage/limits availability for a Codex subscription |
| Local telemetry files (`codex_local_usage`) | Implemented in PoC | Reads `${CODEX_HOME:-~/.codex}` JSONL history, aggregates usage, and reads local `rate_limits` snapshots (`used_percent`, `resets_at`, windows, optional credits) |
| Frontend/dashboard API | Research-only | Possible only with a clear and safe approach to session data |
| Traffic observation | Research-only | Do not consider as a product mechanism |
