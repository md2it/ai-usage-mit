# ai-usage-mit

A small local tracker for AI tool usage and subscription limits on models.

## How it works

For the user, the app acts as a local assistant: it collects available usage and limit data, normalizes it, and shows a clear summary.

1. **user** → **app**: requests limits
2. **app** → **source**: fetches available data
3. **source** → **app**: returns usage/limits/status
4. **app**: normalizes the result
5. **app** → **user**: shows the summary

## Supported features

- **`ai-usage` command** — queries Codex, Claude, and Cursor in one run and prints a normalized usage/limit summary.
- **Codex** — reads local token usage from `${CODEX_HOME:-~/.codex}` and can still read limits via CLI `/status`.
- **Claude** — reads limits via CLI `/usage`.
- **Cursor** — reads usage from `api2.cursor.sh` using a token from `cursor agent login`; if the API is unavailable, falls back to `cursor agent about/status`.

Run from the repository:

```sh
./bin/ai-usage
```

Supported flags are:

- `--help`, `-h`
- `--init-config`
- `--all`, `-a`
- `--codex-local`
- `--codex-cli`
- `--claude-hook`
- `--claude-cli`
- `--claude-local`
- `--cursor-api2`

Show CLI help:

```sh
./bin/ai-usage --help
```

Create a user config:

```sh
./bin/ai-usage --init-config
```

Query only selected sources by passing source flags:

```sh
./bin/ai-usage --codex-local --cursor-api2
```

`--all` and `-a` force all current sources, even when the config defines a narrower default. When no source is selected, the command uses config defaults or, if no config exists, the built-in defaults: Codex local, Claude hook, Cursor API.

Optional config path:

```text
~/.config/ai-usage/config.toml
```

Example:

```toml
default_sources = [
  "codex_local",
  "claude_hook",
  "cursor_api2"
]
```

CLI-backed sources use the standard `codex`, `claude`, and `cursor` CLIs. Local sources read provider files from the user's home directory.
