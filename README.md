# ai-usage-mit

A small local tracker for AI tool usage and subscription limits on models.

## How it works

For the user, the app acts as a local assistant: it collects available usage and limit data, normalizes it, and shows a clear summary.

1. **user** → **app**: requests limits
2. **app** → **source**: fetches available data
3. **source** → **app**: returns usage/limits/status
4. **app**: normalizes the result
5. **app** → **user**: shows the summary

## PoC

The current PoC is the `ai-usage` command, which fetches available usage/limit information for Codex, Claude, and Cursor in one run and exits.

Methods:

- Codex: CLI `/status`.
- Claude: CLI `/usage`.
- Cursor: internal `api2.cursor.sh` via token from `cursor agent login`; if the API is unavailable, falls back to `cursor agent about/status`.

Run from the repository:

```sh
./bin/ai-usage
```

By default, the command uses the standard `codex`, `claude`, and `cursor` CLI tools. You need the relevant provider CLIs installed.
