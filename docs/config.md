# Config

This document describes the `ai-limits` user config file.

## Purpose

The config file lets the user set defaults that apply when no overriding flags are given: which sources to query and how often to repeat a query in watch mode.

## Location

```text
~/.config/ai-limits/config.toml
```

The file is optional. When it does not exist, the command uses built-in defaults.

Create or overwrite it with:

```sh
ai-limits --init-config
```

## Format

The config file is TOML.

```toml
default_sources = [
  "codex_local",
  "claude_local",
  "cursor_api2"
]
watch_interval = "5m"
```

## Keys

### `default_sources`

A list of source identifiers queried when the command runs without explicit source flags and without `--all`.

Valid identifiers: `codex_local`, `codex_cli`, `claude_hook`, `claude_cli`, `claude_local`, `cursor_api2`.

Built-in default, used when the config does not set this key or the config file does not exist:

```toml
default_sources = ["codex_local", "claude_local", "cursor_api2"]
```

`--all`/`-a` ignores `default_sources` and queries every current source.

### `watch_interval`

The interval used by `--watch`/`-w` when the flag is given without a value.

Format: a duration string with a mandatory unit suffix — `s`, `m`, or `h` (for example `"30s"`, `"5m"`, `"1h"`).

Built-in default, used when the config does not set this key or the config file does not exist:

```toml
watch_interval = "5m"
```

`--watch={duration}` overrides `watch_interval` for that run; the config value is not changed.

## Precedence

For each setting, the effective value is resolved in this order:

1. an explicit CLI flag value for that run;
2. the value from the config file;
3. the built-in default.

## Related documents

- [../README.md](../README.md) — flags and usage overview.
- [terminal-ui.md](terminal-ui.md) — watch mode behavior and output format.
