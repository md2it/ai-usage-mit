# Cursor

## Current status

The PoC retrieves numeric Cursor usage/limits through the internal endpoint `api2.cursor.sh` and an access token created by `cursor agent login`.

If the token is not found, the request is rejected, or the response format has changed, the PoC falls back to the standard `cursor` command and the `agent` subcommand. This fallback shows only identity/status/model/tier, because the currently verified Cursor Agent build does not expose numeric usage/limits through a stable CLI command.

Research on `api2.cursor.sh`: [../../references/cursor-api2-cursor-sh.md](../../references/cursor-api2-cursor-sh.md).

---

## Provider Method: `cursor_api2_usage`

The primary PoC method retrieves numeric usage/limits through `api2.cursor.sh`.

The method:

- uses an access token after `cursor agent login`
- calls `GetCurrentPeriodUsage`
- returns included usage, usage percentages, and billing cycle
- depends on an unofficial Cursor backend contract
- requires a separate security review before production use

Endpoint details: [../../references/cursor-api2-cursor-sh.md](../../references/cursor-api2-cursor-sh.md).

---

## Provider Method: `cursor_cli_agent_status`

Minimal commands:

- check CLI availability: `command -v cursor`
- check CLI version: `cursor --version`
- official site: https://cursor.com
- install page: https://cursor.com/install

The fallback PoC method does not return numeric usage/limits, but helps show Cursor Agent state.

Verified details:

- runs the standard `cursor` command and the `agent` subcommand
- no explicit `usage`/`limits` command found in the currently verified Cursor Agent build
- the interactive TUI starts, but does not expose a numeric usage summary through a stable CLI command
- the fallback PoC runs `cursor agent about` and `cursor agent status`
- available data: subscription tier, current model, CLI version, and auth status
- user output explicitly shows that the current CLI build does not return numeric usage/limits

---

## Known usage retrieval options

| Option | Plan/availability | Status | Notes |
|---|---|---|---|
| IDE backend `api2.cursor.sh` | Pro/Ultra/Team | Implemented in PoC | Uses access token after `cursor agent login`; unofficial contract |
| Cursor CLI `about/status` | Pro/Ultra/Team | Fallback in PoC | Provides identity/auth/model/tier, but not billing usage |
| Dashboard API `cursor.com/api/...` | Any | Research-only | Requires web session cookie; high security risk |
| Admin API `api.cursor.com` | Enterprise | Official | Suitable for Enterprise monitoring; 403 expected on Pro/Teams without Enterprise |

---

## Recommendation

For personal Pro/Ultra/Team, the primary PoC option is a locally authorized Cursor Agent and `api2.cursor.sh`. The method remains an unofficial provider method and requires a separate security review before production use.

For production/enterprise monitoring, the official Admin API is preferred when available for the plan and provides the required level of detail.

---

## Limitations

- `api2.cursor.sh` and `cursor.com/api/*` are not publicly documented contracts and may change without notice
- the access token is short-lived
- the refresh token is a sensitive secret
- automated work with dashboard cookies should be disabled by default
