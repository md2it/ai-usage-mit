# Terminal UI

This document describes the actual terminal interface of `ai-usage`.

---

## General Format

Each `ai-usage` response is printed inside a common frame.

Top frame:

```text
=-=-=-=-=-=-=-=-=-=-=-=-= AI USAGE =-=-=-=-=-=-=-=-=-=-=-=-=
```

Bottom frame:

```text
=-=-=-=-=-=-=-=-=-=-=-=-=-= DONE =-=-=-=-=-=-=-=-=-=-=-=-=-=
=-=-=-=-=-=-=-=-=-=-=-=-=-= PART =-=-=-=-=-=-=-=-=-=-=-=-=-=
=-=-=-=-=-=-=-=-=-=-=-=-=-= FAIL =-=-=-=-=-=-=-=-=-=-=-=-=-=
```

An empty line is printed before the top frame, after the top frame, before the bottom frame, and after the bottom frame.

Statuses:

| Status | Meaning |
| --- | --- |
| `DONE` | All requested sources returned a result or a valid unavailable state. |
| `PART` | Some sources returned a result; some ended with an error. |
| `FAIL` | The command did not obtain a usable result. |

---

## Help

`ai-usage --help` uses the common frame.

Format:

```text

=-=-=-=-=-=-=-=-=-=-=-=-= AI USAGE =-=-=-=-=-=-=-=-=-=-=-=-=

Usage:
  ai-usage [OPTIONS]

Options:
  --help, -h      Show this help
  --init-config   Create / overwrite the user config file
  --all, -a       Query all current sources, ignoring config defaults
  --codex-local   Query Codex from local session JSONL files
  --codex-cli     Query Codex through the Codex CLI
  --claude-hook   Query Claude from statusline hook stdin payload
  --claude-cli    Query Claude through the Claude CLI
  --claude-local  Query Claude from local transcript JSONL files
  --cursor-api2   Query Cursor through api2.cursor.sh

Config:
  ~/.config/ai-usage/config.toml

  default_sources = ["codex_local", "claude_hook", "cursor_api2"]


=-=-=-=-=-=-=-=-=-=-=-=-=-= DONE =-=-=-=-=-=-=-=-=-=-=-=-=-=

```

---

## CLI Errors

CLI errors are printed inside the common frame.

Format:

```text

=-=-=-=-=-=-=-=-=-=-=-=-= AI USAGE =-=-=-=-=-=-=-=-=-=-=-=-=

ai-usage: unknown argument `--bad`

=-=-=-=-=-=-=-=-=-=-=-=-=-= FAIL =-=-=-=-=-=-=-=-=-=-=-=-=-=

```

---

## Source Block

Each completed source is printed as a separate block.

Block header:

```text
            ~~~~~~~~~~ CURSOR-API2 ~~~~~~~~~~
            ~~~~~~~~~~ CODEX-CLI ~~~~~~~~~~
            ~~~~~~~~~~ CLAUDE-CLI ~~~~~~~~~~
```

An empty line is printed after the header, then the source result.

Format:

```text
            ~~~~~~~~~~ CURSOR-API2 ~~~~~~~~~~

Cursor usage:
Cursor api2 usage unavailable: token not found; run `cursor agent login`

```

---

## Loader

The loader shows active work for a source and does not show a progress percentage.

Format:

```text
⠋ waiting codex-cli
⠙ waiting claude-cli
```

Unicode spinner frames:

```text
⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏
```

ASCII spinner frames:

```text
- \ | /
```

The ASCII spinner is used when stdout is not a TTY or the environment does not appear to be UTF-8.

The loader starts displaying if a source runs longer than `350ms`.

If a source finishes before the loader is first shown, the loader is not printed.

After a source finishes, the loader is cleared, then the source result block is printed.

---

## Parallel Model

Selected sources are started in parallel.

Execution model:

```text
provider worker threads
        ↓
channel events
        ↓
cli event loop
        ↓
terminal renderer
```

If multiple sources are waiting at the same time, multiple loader lines are displayed.

Format:

```text
⠋ waiting codex-cli
⠙ waiting claude-cli
```

When a source finishes, its loader is cleared and the result is printed as soon as it is ready.

---

## Color

Terminal UI does not use color.

Output does not contain ANSI color codes for frames, headers, the loader, or content.

---

## Loader Cleanup

In an interactive terminal, the loader is redrawn in place.

On each update:

1. previous loader lines are cleared;
2. current loader lines are printed again;
3. the cursor stays in the loader area.

When a source finishes, the loader is cleared before the result is printed.

When `TerminalUi` shuts down, the loader is cleared via `Drop`.

---

## Architectural Boundaries

Layout:

```text
src/cli/mod.rs
  - parses arguments
  - starts provider worker threads
  - receives events via channel
  - passes state to terminal renderer
  - prints source results

src/infra/loader.rs
  - selects unicode/ascii spinner
  - draws loader lines
  - clears loader lines
  - prints frames and headers

src/get_limits.rs
  - calls provider methods
  - returns normalized SourceReport

src/providers/*
  - fetches source data
  - does not render terminal UI
```
