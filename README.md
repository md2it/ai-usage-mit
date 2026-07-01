# ai-limits

Check subscription limits easily. Codex, Claude, Cursor.

## How it works

For the user, the app acts as a local assistant: it collects available usage and limit data, normalizes it, and shows a clear summary.

1. **user** → **app**: requests limits
2. **app** → **source**: fetches available data
3. **source** → **app**: returns usage/limits/status
4. **app**: normalizes the result
5. **app** → **user**: shows the summary

## Supported features

- **`ai-limits` command** — queries Codex, Claude, and Cursor in one run and prints a normalized usage/limit summary.
- **Codex**
  - **local** (`--codex-local`) — reads token usage and local `rate_limits` snapshots from `${CODEX_HOME:-~/.codex}` JSONL files.
  - **CLI** (`--codex-cli`) — reads limits via the Codex CLI `/status` command.
- **Claude**
  - **statusline hook** (`--claude-hook`) — reads live `rate_limits` from Claude Code statusline hook stdin payload (5h/7d windows, reset).
  - **CLI** (`--claude-cli`) — reads limits via the Claude CLI `/usage` command.
  - **local** (`--claude-local`, default) — aggregates token usage history from local transcript JSONL files.
- **Cursor** (`--cursor-api2`) — reads usage from `api2.cursor.sh` using a token from `cursor agent login`; if the API is unavailable, falls back to `cursor agent about/status`.
- **Config** — optional `~/.config/ai-limits/config.toml` with `default_sources`; create with `--init-config`.

Run from the repository:

```sh
./bin/ai-limits
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
./bin/ai-limits --help
```

Create a user config:

```sh
./bin/ai-limits --init-config
```

Query only selected sources by passing source flags:

```sh
./bin/ai-limits --codex-local --cursor-api2
```

`--all` and `-a` force all current sources, even when the config defines a narrower default. When no source is selected, the command uses config defaults or, if no config exists, the built-in defaults: Codex local, Claude local, Cursor API.

Optional config path:

```text
~/.config/ai-limits/config.toml
```

Example:

```toml
default_sources = [
  "codex_local",
  "claude_local",
  "cursor_api2"
]
```

CLI-backed sources use the standard `codex`, `claude`, and `cursor` CLIs. Local sources read provider files from the user's home directory.

## License

This project is licensed under the [MIT License](LICENSE).
