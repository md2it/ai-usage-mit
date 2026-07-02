# Terminal UI

This document describes the actual terminal interface of `ai-limits`.

---

## Help

`ai-limits --help` uses the common frame.

Format:

```text

=-=-=-=-=-=-=-=-=-=-=-=-= AI LIMITS =-=-=-=-=-=-=-=-=-=-=-=-

Usage:
  ai-limits [OPTIONS]

Options:
  --help, -h       Show this help
  --init-config    Create / overwrite the user config file
  --all, -a        Query all current sources, ignoring config defaults
  --usage          Show user-facing usage summary
  --raw, -r        Return raw source data
  --structured, -s Return structured source data
  --watch, -w      Repeat the query on an interval

Technical source options:
  --codex-local    Query Codex from local session JSONL files
  --codex-cli      Query Codex through the Codex CLI
  --claude-hook    Query Claude from statusline hook stdin payload
  --claude-cli     Query Claude through the Claude CLI
  --claude-local   Query Claude from local transcript JSONL files
  --cursor-api2    Query Cursor through api2.cursor.sh

Examples:
  ai-limits --all
  ai-limits --all --usage
  ai-limits --all --raw
  ai-limits --all --structured
  ai-limits --watch
  ai-limits --watch=10m

Config:
  ~/.config/ai-limits/config.toml

  default_sources = ["codex_local", "claude_local", "cursor_api2"]
  watch_interval = "5m"


=-=-=-=-=-=-=-=-=-=-=-=-=-= DONE =-=-=-=-=-=-=-=-=-=-=-=-=-=

```

Default output is the user-facing limits presentation. `--usage` is the user-facing usage presentation. `--raw` and `--structured` are technical output modes for source-level data. They support development, testing, and provider contract checks. `--watch` repeats the default query on an interval; see [Watch Mode](#watch-mode).

Technical source options are working source selectors, but they are primarily intended for intermediate source-level workflows.

---

## UI

### General Format

Each `ai-limits` response is printed inside a common frame.

Top frame:

```text
=-=-=-=-=-=-=-=-=-=-=-=-= AI LIMITS =-=-=-=-=-=-=-=-=-=-=-=-
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

### CLI Errors

CLI errors are printed inside the common frame.

Format:

```text

=-=-=-=-=-=-=-=-=-=-=-=-= AI LIMITS =-=-=-=-=-=-=-=-=-=-=-=-

ai-limits: unknown argument `--bad`

=-=-=-=-=-=-=-=-=-=-=-=-=-= FAIL =-=-=-=-=-=-=-=-=-=-=-=-=-=

```

---

### Provider Block

Default limits output prints each provider as a separate block.

Block header:

```text
            ---------- CODEX ----------
            ---------- CLAUDE ----------
            ---------- CURSOR ----------
```

An empty line is printed before each provider header and after the header.

Each provider block contains:

- zero or more limit rows;
- credits or balance, when available;
- `Source {source}`, using structured `source` and `data_as_of`.

Limit row format:

Format:

```text
{window:<4} {bar:<25} {left:>11} | reset {reset_at}
```

Example:

```text
            ---------- CODEX ----------

5h   ■■□□□□□□□□□□□□□□□□□□□□□□□  8.0% left | reset Jun 30, 21:41 UTC-2
7d   ■■■■■■■■■■■■■■□□□□□□□□□ 54.0% left | reset Jul  3, 21:41 UTC-2
344.2 credits available
Source codex-cli: Jul 3, 21:41 UTC-2
```

The bar width is `25` characters. Each filled bar character `■` represents `4%`. Remaining limit is rounded to the nearest number of filled characters. Empty bar characters use `□`. ANSI color codes in the bar do not affect column width.

Limit rows use fixed visible column widths: `{window}` is 4 characters, `{bar}` is 25 characters, and `{left}` is 11 characters right-aligned. The ` | reset ` separator starts at the same column on every row.

The `{left}` percentage label is always shown with one decimal place (`8.0% left`, `54.0% left`, `62.5% left`). Structured source data may keep finer precision; presentation normalizes the displayed value and uses the same normalized value for the bar and color thresholds.

User-facing timestamps use the local system timezone and are formatted as `{Mon} {day}, {HH:MM} UTC{±offset}` — for example `Jul 3, 22:15 UTC+2`. The day uses a fixed two-character field (`Jul  6, 02:59`, not `Jul 6, 02:59`) so clock times align after the comma across rows. Whole hours omit minutes in the offset (`UTC+2`, not `UTC+2:00`). If a source timestamp cannot be parsed reliably, presentation keeps the original source text.

The filled bar characters show available remaining limit, not used limit. The whole filled part uses one color based on remaining limit. The empty bar characters are not colored.

If `data_as_of` is unavailable, print:

```text
Source codex-cli: unknown
```

If the source is unavailable, print the provider block with the status message:

```text
            ---------- CLAUDE ----------

Unavailable: not logged in
Source claude-cli: unknown
```

If the source is available but has no supported limit data, print the provider block with a short reason:

```text
            ---------- CODEX ----------

No limit data from this source
Source codex-cli: Jul 3, 21:41 UTC-2
```

---

### Usage Block

`--usage` prints each provider as a separate block using the same provider header format as default limits output.

The usage block contains only available user-facing usage facts. Fields with `null` values are not printed.

Example:

```text
            ---------- CODEX ----------

Tokens        input 120k | cached 80k | output 30k | total 230k
Activity      14 sessions | 128 turns | latest Jul 3, 21:41 UTC-2
Models        top: gpt-5
Money         $12.40 used

Source codex-local: Jul 3, 21:41 UTC-2
```

Supported usage rows:

- `Tokens` — input, cached input, output, reasoning output, cache read/write, and total, when available;
- `Activity` — sessions, turns, files, events, and latest activity, when available;
- `Models` — top model, when available;
- `Money` — used, remaining, total, and currency, when available;
- `Source {source}` — structured `source` and `data_as_of`.

If `data_as_of` is unavailable, print:

```text
Source codex-local: unknown
```

If the source is unavailable, use the same unavailable format as default limits output.

---

### Loader

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

### Color

Default output may use color for filled bar characters only.

Frames, headers, loader text, labels, percentages, reset text, and empty bar characters are not colored.

Color is based on remaining limit:

| Remaining limit | Color |
| --- | --- |
| `>= 75%` | green |
| `>= 50%` | yellow |
| `>= 25%` | yellow or orange, depending on terminal support |
| `>= 10%` | red |
| `< 10%` | bright red |

Color is optional. If stdout is not a TTY, the terminal does not support color, or color is disabled by environment settings such as `NO_COLOR`, output must remain readable without ANSI color codes.

---

### Loader Cleanup

In an interactive terminal, the loader is redrawn in place.

On each update:

1. previous loader lines are cleared;
2. current loader lines are printed again;
3. the cursor stays in the loader area.

When a source finishes, the loader is cleared before the result is printed.

When `TerminalUi` shuts down, the loader is cleared via `Drop`.

---

## Watch Mode

`--watch`/`-w` repeats the query on a fixed interval instead of running once. `--watch={duration}` (for example `--watch=10m`) sets the interval for that run; without a value, the interval comes from the config.

Each cycle prints a full response inside the common frame, exactly as a single default run would. Cycles repeat until the process is interrupted (`Ctrl+C`).

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

## Architectural Boundaries

Layout:

```text
src/cli/mod.rs
  - parses arguments
  - starts provider worker threads
  - receives events via channel
  - passes state to terminal renderer
  - prints provider presentation results

src/presentation/*
  - converts structured data into user-facing provider blocks
  - selects display labels, limit rows, bar values, and fallback messages
  - does not fetch source data

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
  - returns raw and structured data
  - does not render terminal UI
```
