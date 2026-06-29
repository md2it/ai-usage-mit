# Getting Limits From Statusline Hook

This document describes provider methods that fetch live usage/limits from statusline hook stdin payloads.

---

## Base Flow

The diagram below describes the general process for a provider method that reads hook stdin payload.

```mermaid
sequenceDiagram
    actor User as User
    participant App as Application
    participant Hook as Provider statusline hook

    User->>App: Requests limits
    App->>App: Selects statusline-hook method
    App->>Hook: Executes hook command in supported context
    Hook-->>App: Sends JSON payload on stdin
    App->>App: Extracts rate_limits and related fields
    App->>App: Normalizes and validates live data
    App-->>User: Shows current limits and reset
```

---

## Hook Runtime Context

The diagram below describes context requirements for hook-based methods.

```mermaid
stateDiagram-v2
    [*] --> Limit_request

    Limit_request --> Provider_selected
    Provider_selected --> Method_selected
    Method_selected --> Hook_context_check

    Hook_context_check --> Hook_available: Supported hook context
    Hook_context_check --> Hook_unavailable: Unsupported context

    Hook_available --> Read_stdin_payload
    Read_stdin_payload --> Parse_rate_limits
    Parse_rate_limits --> Normalize_data
    Normalize_data --> Limits_shown_to_user

    Hook_unavailable --> Limits_unavailable

    Limits_shown_to_user --> [*]
    Limits_unavailable --> [*]
```

---

## Rules

- this method is valid only for providers that expose hook payload on stdin
- hook payload parsing must be strict for required fields and tolerant for optional fields
- normalization must prioritize provider live fields over reconstructed estimates
- if payload contains `rate_limits`, output should treat it as the primary live signal
- if payload omits `rate_limits`, output must mark live limits as unavailable
- hook methods must not require TUI parsing when hook payload already includes structured fields
- method behavior must be deterministic for unsupported contexts

---

## Configuration Requirements

- provider-specific statusline command must be configured in the provider settings file
- hook command must run in the provider-supported statusline context
- stdin payload format and required fields must be documented per provider
- security-sensitive tokens from hook payload must not be persisted unless explicitly required and approved

---

## Deviations From the Flow

- if hook context is not available, the application shows a clear unavailable state and fallback option
- if stdin is empty or invalid JSON, the application shows an appropriate parse error
- if payload exists but lacks limit fields, the application shows usage context and marks limits as unavailable
- if provider changes payload schema, the application must fail safely and report unsupported schema
