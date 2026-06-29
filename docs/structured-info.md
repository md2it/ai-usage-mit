# STRUCTURED INFO

## Purpose

This document defines the expected output of data processing.

## Input

For each information source, the system has a data collection method documented in [docs/get-info](get-info/).

Each source may have its own request format, access method, raw response format, limitations, and fallback path.

## Output modes

The same source scripts should provide two output modes:

1. Raw data

   All data that the source script received or extracted from the underlying source, without product-level structuring.

   For a CLI source, this is the captured CLI output. For an API source, this is the response received from the API. For a local-file source, this is the data that the source script read or extracted according to its collection method.

   Raw data does not have to be stable between runs and does not have to follow a common schema.

2. Structured data

   Data converted into the common product-level structure defined below.

   Structured data must be stable and machine-readable. It must follow the same field contract for every provider and source.

User-facing presentation is a separate layer. Source scripts should not define the final terminal summary format, limit bars, colors, provider headers, or fallback display text.

## Structured format

Structured data should use one common field contract for all providers and sources.

The current minimum structure is described below in a YAML-like schema. The field names, nesting, and meanings are mandatory. The final serialization format may be selected by the implementation as long as it remains machine-readable and preserves this structure.

```yaml
provider: string
source: string
source_link: docs/get-info
status:
  data_available: boolean
  access_available: boolean
  message: string | null
raw_data_available: boolean
collected_at: string | null
data_as_of: string | null
account:
  plan: string | null
  credits_total: number | null
  credits_used: number | null
  credits_remaining: number | null
limits:
  - name: string
    window_label: string | null
    window_minutes: number | null
    resets_at: string | null
    used_percent: number | null
    remaining_percent: number | null
    used_amount: number | null
    remaining_amount: number | null
    total_amount: number | null
    amount_unit: string | null
usage:
  tokens:
    input: number | null
    cached_input: number | null
    output: number | null
    reasoning_output: number | null
    cache_read: number | null
    cache_write: number | null
    total: number | null
  money:
    used_amount: number | null
    remaining_amount: number | null
    total_amount: number | null
    currency: string | null
  activity:
    events_count: number | null
    files_count: number | null
    sessions_count: number | null
    turns_count: number | null
    latest_activity_at: string | null
  models:
    top_model: string | null
diagnostics:
  - string
```

## Limit rules

Limits must be represented consistently, even when providers report them differently.

If a source provides `used_percent`, the system should also calculate `remaining_percent` when possible.

If a source provides `remaining_percent`, the system should also calculate `used_percent` when possible.

If a source provides used, remaining, and total amounts, all available values should be preserved.

If only two amount values are available and the third can be calculated reliably, the system should calculate it.

`amount_unit` should describe what is being limited, for example `tokens`, `credits`, `usd`, `requests`, or another provider-specific unit.

## Time fields

`collected_at` is the time when `ai-limits` collected or read the source data.

`data_as_of` is the time when the source data itself was last current. For local files, transcripts, or hook payloads, this is usually the timestamp of the latest relevant source record or session. For live API or CLI responses, this may be the response or snapshot time.

The default terminal presentation uses `data_as_of` for the `Data as of` line. It does not use `collected_at` for this line.

`usage.activity.latest_activity_at` is a separate business fact about user activity. It must not be treated as the default `Data as of` value unless it is also the best known timestamp for the source data itself.

## Empty and unavailable values

If a value is not present in the source data, use `null`.

If a value exists but cannot be parsed reliably, use `null` and add a short explanation to `diagnostics`.

If a value can be calculated only by making a weak assumption, do not calculate it. Use `null` and add a short explanation to `diagnostics`.

If the source cannot be accessed, set `status.access_available` to `false`, `status.data_available` to `false`, and put the user-readable reason into `status.message`.

If the source is accessible but does not contain supported usage or limit data, set `status.access_available` to `true`, `status.data_available` to `false`, and put the reason into `status.message`.

If raw data can be returned for the source, set `raw_data_available` to `true`. If the implementation cannot expose raw data safely or technically, set it to `false`.
