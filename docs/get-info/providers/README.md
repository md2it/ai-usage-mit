# PROVIDER CONTRACT

## Purpose

This document defines the implementation contract for `src/providers/*`.

## Source result

Each provider source method should be able to return two source-level results:

1. Raw data

   The full data captured, received, read, or extracted by the source method.

2. Structured data

   The same source data converted to the common contract from [docs/structured-info.md](structured-info.md).

## Rules

- Provider modules may contain source-specific parsing and normalization.
- Provider modules must not define the final user-facing terminal summary.
- Structured data should be represented in Rust types, not as ad hoc formatted strings.
- Raw data should remain available until structured data is produced.
- Missing, unavailable, or unreliable values must follow [docs/structured-info.md](structured-info.md).
- Source failures that are meaningful to the user should be represented as status data when possible.

## Tests

Each source should have focused tests for:

- structured data built from a representative source sample;
- unavailable or access-denied source state;
- source-specific parsing rules that affect limits or usage.
