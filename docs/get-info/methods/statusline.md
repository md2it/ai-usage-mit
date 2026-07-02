# Statusline Runtime Capture

## Purpose

Statusline runtime capture retrieves live provider data that is available only while a local tool is running. The tool calls a configured command, sends a JSON payload through stdin, and the command saves the latest valid payload to a local cache.

This method is separate from static local-file scanning. Static scanning reads files that the provider already writes for history. Statusline capture asks the provider runtime to pass a fresh snapshot to a configured local command.

## Claude Code

Claude Code supports a `statusLine` command in `~/.claude/settings.json` or `~/.config/claude/settings.json`.

Claude Code passes statusline input as JSON on stdin. For Claude.ai Pro/Max accounts, the payload includes `rate_limits` after a Claude API response is available.

Observed useful fields:

- `rate_limits.five_hour.used_percentage`
- `rate_limits.five_hour.resets_at`
- `rate_limits.seven_day.used_percentage`
- `rate_limits.seven_day.resets_at`

## Capture Model

1. Claude Code receives or refreshes live account/session data.
2. Claude Code starts the configured statusline command.
3. Claude Code writes the JSON payload to the command stdin.
4. The command validates that `rate_limits` exists.
5. The command writes the payload to an ai-limits cache file.
6. ai-limits reads the cache file later.

The capture command is not a daemon. It runs briefly, saves a snapshot, and exits.

## Data Quality

- `rate_limits` is the strongest local signal found for Claude Code current limits.
- `resets_at` is a server-provided reset timestamp, not a transcript reconstruction.
- The cache is only as fresh as the latest statusline payload.
- The cache can be missing before the first Claude response after setup.
- The cache can be stale when Claude is used on another device or another surface that does not run the configured statusline.

## Boundaries

- This method is confirmed for Claude Code CLI statusline behavior.
- It is not confirmed for Claude Desktop, browser extension, or web-only Claude usage.
- It does not provide historical usage by itself.
- It does not replace transcript scanning for token history.
