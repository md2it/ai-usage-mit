# Architecture

This document defines the target structure of `src/` after moving from a PoC monolith to a maintainable application.

---

## Goal

The code should support:

- a CLI interface
- a future desktop interface
- multiple providers
- multiple ways to fetch data for a single provider
- small files with a clear area of responsibility

The CLI and the future desktop should share a common core, not separate business logic.

---

## `src/` Structure

Target structure for the near term:

```text
src/
  cli/
  config/
  infra/
  providers/
  get_limits.rs
  lib.rs
  types.rs
```

Purpose:

- `cli/` — terminal interface, arguments, output, exit codes
- `config/` — user settings, defaults, and paths to config files
- `infra/` — technical primitives for processes, HTTP, and timeouts
- `providers/` — ways to fetch usage/limits from providers
- `get_limits.rs` — limits-fetching scenario and provider method integration
- `lib.rs` — shared core available to different interfaces
- `types.rs` — shared types and the application's internal language

---

## Boundaries

Module rules:

- `cli/` does not fetch data from providers directly
- `cli/` calls the shared core and is responsible only for terminal behavior
- `get_limits.rs` coordinates config, providers, and fallback logic
- `get_limits.rs` does not run processes or HTTP directly when that can be delegated to provider/infra
- `providers/` does not format terminal output
- `providers/` returns normalized types from `types.rs`
- `infra/` does not know the business meaning of usage/limits
- `infra/` is responsible only for technical interaction with the outside world
- `types.rs` must not depend on CLI, desktop, the file system, or external commands

---

## Providers

Initially, `providers/` remains a flat directory.

Example:

```text
providers/
  mod.rs
  codex_cli_usage.rs
  claude_cli_usage.rs
  cursor_api2_usage.rs
```

Rules:

- one file describes one way to fetch data
- each data-fetching method must be independent of the others
- removing one method must not break the rest
- shared technical logic goes in `infra/`
- shared business types go in `types.rs`

If a single provider grows to many files, you can move to a nested structure by provider.

---

## `get_limits` Scenario

`get_limits.rs` follows the document [get-info/methods/README.md](get-info/methods/README.md).

Purpose:

- select enabled provider methods
- call provider methods in the right order
- apply fallback logic
- assemble a shared result for the CLI and the future desktop

Boundaries:

- does not contain terminal output
- does not contain low-level process execution
- does not contain low-level HTTP primitives
- does not parse provider-specific output when that is a provider method's responsibility

---

## Provider Specs

Provider documentation is grouped by provider:

```text
docs/get-info/providers/
  codex.md
  claude.md
  cursor.md
```

Rules:

- one spec file describes one provider
- a spec file may describe multiple provider methods
- provider method sections are named like future code files without `.rs`
- code may be more detailed than the documentation and split provider methods into separate files
- if a spec file becomes too large, it can be split by provider method

---

## Configuration

User settings must not be baked into the compiled binary.

Model:

- defaults live in code
- user config is stored in a separate runtime file
- the CLI and the future desktop use the same config
- platform-specific config file paths are defined inside `config/`

---

## Future Desktop

The desktop will almost certainly be built with Tauri, but a separate desktop directory is not created yet.

Current rule:

- the shared core must live in `lib.rs` and the `src/` modules
- the CLI must be only one interface to the core
- Tauri integration should appear later as a separate interface to the same core

---

## Notifications

The `notifications/` directory is not created yet, until system notifications are implemented.

Future rule:

- notifications should be a shared service, not part of desktop only
- the CLI can use notifications if the platform supports it and it is enabled in config
- platform differences must be isolated inside the notifications module

---

## Rule for Agents

When making changes, first identify the business area of the task:

- terminal behavior — `cli/`
- settings — `config/`
- data fetching — `providers/`
- limits-fetching scenario — `get_limits.rs`
- process execution, HTTP, timeouts — `infra/`
- shared data structures — `types.rs`

If a task spans more than one area, describe the overlap explicitly before making changes.
