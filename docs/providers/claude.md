# Claude

## Current Status

PoC uses two Claude sources and one live-limit candidate:

- `claude_cli_usage`: launches `claude --no-chrome`, sends `/usage`, parses TUI lines.
- `claude_local_usage`: scans local transcript JSONL files and aggregates token usage history.
- `claude_statusline_rate_limits`: reads Claude Code hook stdin payload and extracts live `rate_limits` when available.

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

---

## Provider Method: `claude_statusline_rate_limits`

Minimal source:

- Claude Code statusline hook stdin payload
- no TUI parsing and no transcript reconstruction for current limits

How to get data:

1. configure a Claude Code statusline command in `~/.claude/settings.json` or `~/.config/claude/settings.json`
2. run the command in statusline hook context so Claude Code provides JSON payload on stdin
3. parse `rate_limits` from stdin payload
4. normalize available live fields for current windows (5h/7d), used progress, and reset time

Behavior:

- when hook payload includes `rate_limits`, this method can provide an official live signal for current Claude limits
- when hook context is unavailable or payload has no `rate_limits`, method returns unavailable/unknown for live limits
- this method is for current live limits, not full historical usage aggregation

---

## Limitations

- for `claude_cli_usage`, full output remains a TUI stream and depends on current CLI text/layout
- for `claude_cli_usage`, request/parse can take noticeable time
- for `claude_statusline_rate_limits`, data is available only inside a properly configured Claude Code hook context
- for `claude_statusline_rate_limits`, unavailable hook context means live limits are unavailable even if transcript history exists

---

## Other Options

| Option | Status | Comment |
|---|---|---|
| Official API | Not investigated | May apply to API accounts, but not necessarily to Claude Code subscription limits |
| Local transcript JSONL (`claude_local_usage`) | Implemented in PoC | Scans local transcript roots and aggregates token usage history by assistant turns; official remaining limit/reset may be unavailable |
| Claude Code statusline `rate_limits` | Candidate for live limits | Hook receives JSON via stdin from Claude Code and can provide an official live signal for 5h/7d limits; requires statusline configuration |
| Local SQLite/cache | Auxiliary layer | e.g. `~/.claude/usage.db` from `claude-usage`: convenient for dashboard and incremental scanning, but this is derived data, not a primary source |
| Frontend/dashboard API | Research-only | Possible only with a clear and safe way to handle cookie/session tokens |
| Traffic observation | Research-only | Not to be considered as a product mechanism |
