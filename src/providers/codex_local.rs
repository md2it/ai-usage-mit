use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::{
    AccountInfo, ActivityUsage, LimitInfo, SourceData, SourceStatus, StructuredSourceInfo,
    TokenUsage, UsageInfo,
};

const PROVIDER: &str = "codex";
const SOURCE: &str = "codex_local";
const SOURCE_LINK: &str = "docs/get-info";

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CodexLocalRaw {
    pub root: String,
    pub files_scanned: u64,
    pub token_events: u64,
    pub totals: CodexLocalTokenTotals,
    pub latest_timestamp: Option<String>,
    pub latest_rate_limits_timestamp: Option<String>,
    pub latest_rate_limits: Option<CodexLocalRateLimits>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CodexLocalTokenTotals {
    pub input_tokens: u64,
    pub cached_input_tokens: u64,
    pub output_tokens: u64,
    pub reasoning_output_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CodexLocalRateLimits {
    pub primary: Option<CodexLocalRateLimitWindow>,
    pub secondary: Option<CodexLocalRateLimitWindow>,
    pub credits: Option<f64>,
    pub credits_unlimited: bool,
    pub plan_type: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CodexLocalRateLimitWindow {
    pub used_percent: Option<f64>,
    pub window_minutes: Option<u64>,
    pub resets_at: Option<u64>,
}

#[derive(Default)]
struct CodexLocalUsage {
    files_scanned: u64,
    token_events: u64,
    totals: CodexLocalTokenTotals,
    latest_timestamp: Option<String>,
    latest_rate_limits_timestamp: Option<String>,
    latest_rate_limits: Option<CodexLocalRateLimits>,
}

struct TokenEvent {
    timestamp: Option<String>,
    usage: Option<CodexLocalTokenTotals>,
    rate_limits: Option<CodexLocalRateLimits>,
}

pub fn get_usage() -> io::Result<SourceData> {
    collect()
}

pub fn collect() -> io::Result<SourceData> {
    let root = codex_home()?;
    let collected_at = Some(Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string());

    if !root.exists() {
        let raw = CodexLocalRaw {
            root: root.display().to_string(),
            ..CodexLocalRaw::default()
        };
        let structured = build_structured(
            &raw,
            collected_at,
            false,
            false,
            Some(format!("not found: {}", root.display())),
        );
        return Ok(source_data_from_raw(&raw, structured));
    }

    let mut usage = CodexLocalUsage::default();
    scan_dir(&root.join("sessions"), &mut usage)?;
    scan_dir(&root.join("archived_sessions"), &mut usage)?;

    let raw = raw_from_usage(&root, &usage);
    let (data_available, message) = if usage.token_events == 0 {
        (false, Some("token events: not found".to_string()))
    } else {
        (true, None)
    };
    let structured = build_structured(&raw, collected_at, true, data_available, message);

    Ok(source_data_from_raw(&raw, structured))
}

pub fn decode_raw(raw: Option<&str>) -> Option<CodexLocalRaw> {
    raw.and_then(|value| serde_json::from_str(value).ok())
}

fn source_data_from_raw(raw: &CodexLocalRaw, structured: StructuredSourceInfo) -> SourceData {
    SourceData {
        raw: serde_json::to_string(raw).ok(),
        structured,
        stderr: String::new(),
    }
}

fn raw_from_usage(root: &Path, usage: &CodexLocalUsage) -> CodexLocalRaw {
    CodexLocalRaw {
        root: root.display().to_string(),
        files_scanned: usage.files_scanned,
        token_events: usage.token_events,
        totals: usage.totals.clone(),
        latest_timestamp: usage.latest_timestamp.clone(),
        latest_rate_limits_timestamp: usage.latest_rate_limits_timestamp.clone(),
        latest_rate_limits: usage.latest_rate_limits.clone(),
    }
}

fn build_structured(
    raw: &CodexLocalRaw,
    collected_at: Option<String>,
    access_available: bool,
    data_available: bool,
    message: Option<String>,
) -> StructuredSourceInfo {
    let mut diagnostics = Vec::new();
    let mut limits = Vec::new();

    if let Some(rate_limits) = &raw.latest_rate_limits {
        if let Some(primary) = &rate_limits.primary {
            limits.push(limit_from_window("primary", primary));
        }
        if let Some(secondary) = &rate_limits.secondary {
            limits.push(limit_from_window("secondary", secondary));
        }
        if rate_limits.credits_unlimited {
            diagnostics.push("credits: unlimited".to_string());
        }
    } else if data_available {
        diagnostics.push("limits/reset: unavailable in local Codex JSONL".to_string());
    }

    let account = account_from_rate_limits(raw.latest_rate_limits.as_ref());
    let usage = usage_from_raw(raw);
    let data_as_of = raw
        .latest_rate_limits_timestamp
        .clone()
        .or_else(|| raw.latest_timestamp.clone());
    if data_available && data_as_of.is_none() {
        diagnostics.push("latest source record timestamp is unavailable".to_string());
    }

    StructuredSourceInfo {
        provider: PROVIDER.to_string(),
        source: SOURCE.to_string(),
        source_link: SOURCE_LINK.to_string(),
        status: SourceStatus {
            data_available,
            access_available,
            message,
        },
        raw_data_available: true,
        collected_at,
        data_as_of,
        account,
        limits,
        usage,
        diagnostics,
    }
}

fn account_from_rate_limits(rate_limits: Option<&CodexLocalRateLimits>) -> AccountInfo {
    let Some(rate_limits) = rate_limits else {
        return AccountInfo::default();
    };

    AccountInfo {
        plan: rate_limits.plan_type.clone(),
        credits_total: None,
        credits_used: None,
        credits_remaining: if rate_limits.credits_unlimited {
            None
        } else {
            rate_limits.credits
        },
    }
}

fn usage_from_raw(raw: &CodexLocalRaw) -> UsageInfo {
    let has_tokens = raw.token_events > 0;
    UsageInfo {
        tokens: if has_tokens {
            TokenUsage {
                input: Some(raw.totals.input_tokens),
                cached_input: Some(raw.totals.cached_input_tokens),
                output: Some(raw.totals.output_tokens),
                reasoning_output: Some(raw.totals.reasoning_output_tokens),
                cache_read: None,
                cache_write: None,
                total: Some(raw.totals.total_tokens),
            }
        } else {
            TokenUsage::default()
        },
        activity: ActivityUsage {
            events_count: Some(raw.token_events),
            files_count: Some(raw.files_scanned),
            sessions_count: None,
            turns_count: None,
            latest_activity_at: raw
                .latest_rate_limits_timestamp
                .clone()
                .or_else(|| raw.latest_timestamp.clone()),
        },
        ..UsageInfo::default()
    }
}

fn limit_from_window(name: &str, window: &CodexLocalRateLimitWindow) -> LimitInfo {
    let remaining_pct = window.used_percent.map(calc_remaining_percent);

    LimitInfo {
        name: name.to_string(),
        window_label: window.window_minutes.map(window_label),
        window_minutes: window.window_minutes,
        resets_at: window.resets_at.map(format_unix_utc),
        used_percent: window.used_percent,
        remaining_percent: remaining_pct,
        used_amount: None,
        remaining_amount: None,
        total_amount: None,
        amount_unit: None,
    }
}

fn window_label(minutes: u64) -> String {
    match minutes {
        300 => "5h (300m)".to_string(),
        10080 => "weekly (10080m)".to_string(),
        _ => format!("{minutes}m"),
    }
}

fn calc_remaining_percent(used_percent: f64) -> f64 {
    let raw = (100.0 - used_percent).max(0.0);
    (raw * 10.0).round() / 10.0
}

fn codex_home() -> io::Result<PathBuf> {
    if let Some(value) = env::var_os("CODEX_HOME") {
        return Ok(PathBuf::from(value));
    }

    let home = env::var_os("HOME").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "HOME is not set; cannot locate ${CODEX_HOME:-~/.codex}",
        )
    })?;

    Ok(PathBuf::from(home).join(".codex"))
}

fn scan_dir(path: &Path, usage: &mut CodexLocalUsage) -> io::Result<()> {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            scan_dir(&path, usage)?;
        } else if file_type.is_file() && path.extension().is_some_and(|ext| ext == "jsonl") {
            scan_file(&path, usage)?;
        }
    }

    Ok(())
}

fn scan_file(path: &Path, usage: &mut CodexLocalUsage) -> io::Result<()> {
    usage.files_scanned += 1;

    let content = fs::read_to_string(path)?;

    for line in content.lines() {
        let Some(event) = parse_token_event(line) else {
            continue;
        };

        usage.token_events += 1;

        if let Some(tokens) = &event.usage {
            usage.totals.input_tokens += tokens.input_tokens;
            usage.totals.cached_input_tokens += tokens.cached_input_tokens;
            usage.totals.output_tokens += tokens.output_tokens;
            usage.totals.reasoning_output_tokens += tokens.reasoning_output_tokens;
            usage.totals.total_tokens += tokens.total_tokens;
        }

        if let Some(timestamp) = event.timestamp {
            if usage
                .latest_timestamp
                .as_ref()
                .is_none_or(|latest| timestamp > *latest)
            {
                usage.latest_timestamp = Some(timestamp.clone());
            }

            if event.rate_limits.is_some()
                && usage
                    .latest_rate_limits_timestamp
                    .as_ref()
                    .is_none_or(|latest| timestamp > *latest)
            {
                usage.latest_rate_limits_timestamp = Some(timestamp);
                usage.latest_rate_limits = event.rate_limits;
            }
        }
    }

    Ok(())
}

fn parse_token_event(line: &str) -> Option<TokenEvent> {
    let record = serde_json::from_str::<Value>(line).ok()?;
    if !is_token_count_event(&record) {
        return None;
    }

    let usage = parse_token_usage(&record);
    let rate_limits = parse_rate_limits(&record);

    if usage.is_none() && rate_limits.is_none() {
        return None;
    }

    Some(TokenEvent {
        timestamp: record
            .get("timestamp")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        usage,
        rate_limits,
    })
}

fn parse_token_usage(record: &Value) -> Option<CodexLocalTokenTotals> {
    let usage_value = record
        .get("last_token_usage")
        .or_else(|| record.pointer("/payload/info/last_token_usage"))?;

    Some(CodexLocalTokenTotals {
        input_tokens: number_u64(usage_value, "input_tokens")?,
        cached_input_tokens: number_u64(usage_value, "cached_input_tokens").unwrap_or(0),
        output_tokens: number_u64(usage_value, "output_tokens")?,
        reasoning_output_tokens: number_u64(usage_value, "reasoning_output_tokens").unwrap_or(0),
        total_tokens: number_u64(usage_value, "total_tokens")?,
    })
}

fn is_token_count_event(record: &Value) -> bool {
    record.get("type").and_then(Value::as_str) == Some("token_count")
        || (record.get("type").and_then(Value::as_str) == Some("event_msg")
            && record.pointer("/payload/type").and_then(Value::as_str) == Some("token_count"))
}

fn parse_rate_limits(record: &Value) -> Option<CodexLocalRateLimits> {
    let value = record
        .get("rate_limits")
        .or_else(|| record.pointer("/payload/rate_limits"))?;

    let (credits, credits_unlimited) = parse_credits(value.get("credits"));

    Some(CodexLocalRateLimits {
        primary: parse_rate_limit_window(value.get("primary")),
        secondary: parse_rate_limit_window(value.get("secondary")),
        credits,
        credits_unlimited,
        plan_type: value
            .get("plan_type")
            .and_then(Value::as_str)
            .map(ToString::to_string),
    })
}

fn parse_credits(value: Option<&Value>) -> (Option<f64>, bool) {
    let Some(value) = value else {
        return (None, false);
    };

    if let Some(number) = number_f64_any(value) {
        return (Some(number), false);
    }

    let Some(object) = value.as_object() else {
        return (None, false);
    };

    if object.get("has_credits").and_then(Value::as_bool) == Some(false) {
        return (None, false);
    }

    if object.get("unlimited").and_then(Value::as_bool) == Some(true) {
        return (None, true);
    }

    (object.get("balance").and_then(number_f64_any), false)
}

fn parse_rate_limit_window(value: Option<&Value>) -> Option<CodexLocalRateLimitWindow> {
    let value = value?;
    let used_percent = value.get("used_percent").and_then(number_f64_any);
    let window_minutes = value.get("window_minutes").and_then(number_u64_any);
    let resets_at = value.get("resets_at").and_then(number_u64_any);

    if used_percent.is_none() && window_minutes.is_none() && resets_at.is_none() {
        return None;
    }

    Some(CodexLocalRateLimitWindow {
        used_percent,
        window_minutes,
        resets_at,
    })
}

fn number_u64(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(number_u64_any)
}

fn number_u64_any(value: &Value) -> Option<u64> {
    if let Some(number) = value.as_u64() {
        return Some(number);
    }
    value.as_str().and_then(|raw| raw.parse::<u64>().ok())
}

fn number_f64_any(value: &Value) -> Option<f64> {
    if let Some(number) = value.as_f64() {
        return Some(number);
    }
    value.as_str().and_then(|raw| raw.parse::<f64>().ok())
}

fn format_unix_utc(seconds: u64) -> String {
    DateTime::<Utc>::from_timestamp(seconds as i64, 0)
        .map(|value| value.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        .unwrap_or_else(|| format!("{seconds} (unix)"))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn fixture_path(suffix: &str) -> PathBuf {
        env::temp_dir().join(format!(
            "ai-limits-codex-local-{}-{suffix}.jsonl",
            std::process::id()
        ))
    }

    fn scan_fixture(content: &str, suffix: &str) -> CodexLocalRaw {
        let path = fixture_path(suffix);
        fs::write(&path, content).expect("write fixture");

        let mut usage = CodexLocalUsage::default();
        scan_file(&path, &mut usage).expect("scan fixture");
        let raw = raw_from_usage(Path::new("/tmp/.codex"), &usage);
        let _ = fs::remove_file(&path);
        raw
    }

    #[test]
    fn builds_structured_data_from_representative_sample() {
        let raw = scan_fixture(
            r#"{"type":"event_msg","timestamp":"2026-06-28T10:00:00Z","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":10,"cached_input_tokens":2,"output_tokens":3,"reasoning_output_tokens":1,"total_tokens":16}},"rate_limits":{"primary":{"used_percent":12.4,"window_minutes":300,"resets_at":1750000000}}}}
{"type":"event_msg","timestamp":"2026-06-28T11:00:00Z","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":20,"cached_input_tokens":4,"output_tokens":6,"reasoning_output_tokens":2,"total_tokens":32}},"rate_limits":{"primary":{"used_percent":"45","window_minutes":"300","resets_at":"1750003600"},"secondary":{"used_percent":71.9,"window_minutes":10080,"resets_at":1750600000},"credits":123.6,"plan_type":"pro"}}}
"#,
            "structured",
        );

        let structured = build_structured(&raw, None, true, true, None);

        assert_eq!(structured.provider, "codex");
        assert_eq!(structured.source, "codex_local");
        assert_eq!(structured.source_link, "docs/get-info");
        assert!(structured.status.access_available);
        assert!(structured.status.data_available);
        assert!(structured.raw_data_available);
        assert_eq!(raw.token_events, 2);
        assert_eq!(structured.usage.tokens.input, Some(30));
        assert_eq!(structured.usage.tokens.cached_input, Some(6));
        assert_eq!(structured.usage.tokens.output, Some(9));
        assert_eq!(structured.usage.tokens.reasoning_output, Some(3));
        assert_eq!(structured.usage.tokens.total, Some(48));
        assert_eq!(structured.usage.activity.events_count, Some(2));
        assert_eq!(structured.usage.activity.files_count, Some(1));
        assert_eq!(
            structured.usage.activity.latest_activity_at.as_deref(),
            Some("2026-06-28T11:00:00Z")
        );
        assert_eq!(
            structured.data_as_of.as_deref(),
            Some("2026-06-28T11:00:00Z")
        );
        assert_eq!(structured.account.plan.as_deref(), Some("pro"));
        assert_eq!(structured.account.credits_remaining, Some(123.6));

        let primary = structured
            .limits
            .iter()
            .find(|limit| limit.name == "primary")
            .expect("primary limit");
        assert_eq!(primary.used_percent, Some(45.0));
        assert_eq!(primary.remaining_percent, Some(55.0));
        assert_eq!(primary.window_minutes, Some(300));
        assert_eq!(primary.window_label.as_deref(), Some("5h (300m)"));
        assert_eq!(primary.resets_at.as_deref(), Some("2025-06-15T16:06:40Z"));

        let secondary = structured
            .limits
            .iter()
            .find(|limit| limit.name == "secondary")
            .expect("secondary limit");
        assert_eq!(secondary.used_percent, Some(71.9));
        assert_eq!(secondary.remaining_percent, Some(28.1));
        assert_eq!(secondary.window_label.as_deref(), Some("weekly (10080m)"));
    }

    #[test]
    fn structured_marks_missing_root_as_inaccessible() {
        let raw = CodexLocalRaw {
            root: "/missing/.codex".to_string(),
            ..CodexLocalRaw::default()
        };
        let structured = build_structured(
            &raw,
            None,
            false,
            false,
            Some("not found: /missing/.codex".to_string()),
        );

        assert!(!structured.status.access_available);
        assert!(!structured.status.data_available);
        assert_eq!(
            structured.status.message.as_deref(),
            Some("not found: /missing/.codex")
        );
        assert!(structured.limits.is_empty());
        assert_eq!(structured.usage.tokens.total, None);
    }

    #[test]
    fn structured_marks_accessible_root_without_token_events() {
        let raw = CodexLocalRaw {
            root: "/tmp/.codex".to_string(),
            files_scanned: 3,
            ..CodexLocalRaw::default()
        };
        let structured = build_structured(
            &raw,
            None,
            true,
            false,
            Some("token events: not found".to_string()),
        );

        assert!(structured.status.access_available);
        assert!(!structured.status.data_available);
        assert_eq!(
            structured.status.message.as_deref(),
            Some("token events: not found")
        );
        assert_eq!(structured.usage.activity.files_count, Some(3));
        assert_eq!(structured.usage.activity.events_count, Some(0));
    }

    #[test]
    fn keeps_limits_from_latest_rate_limits_event_not_latest_event() {
        let raw = scan_fixture(
            r#"{"type":"event_msg","timestamp":"2026-06-28T10:00:00Z","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":10,"cached_input_tokens":0,"output_tokens":5,"reasoning_output_tokens":0,"total_tokens":15}},"rate_limits":{"primary":{"used_percent":10,"window_minutes":300,"resets_at":1750000000}}}}
{"type":"event_msg","timestamp":"2026-06-28T12:00:00Z","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":5,"cached_input_tokens":0,"output_tokens":2,"reasoning_output_tokens":0,"total_tokens":7}}}}
"#,
            "latest-limits",
        );

        let structured = build_structured(&raw, None, true, true, None);
        let primary = structured
            .limits
            .iter()
            .find(|limit| limit.name == "primary")
            .expect("primary limit");

        assert_eq!(primary.used_percent, Some(10.0));
        assert_eq!(
            structured.usage.activity.latest_activity_at.as_deref(),
            Some("2026-06-28T10:00:00Z")
        );
        assert_eq!(raw.totals.total_tokens, 22);
    }

    #[test]
    fn accepts_rate_limits_only_token_count_with_null_info() {
        let raw = scan_fixture(
            r#"{"type":"event_msg","timestamp":"2026-06-29T01:46:39.473Z","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":10,"cached_input_tokens":0,"output_tokens":5,"reasoning_output_tokens":0,"total_tokens":15}},"rate_limits":{"primary":{"used_percent":86.0,"window_minutes":300,"resets_at":1782709162},"plan_type":"plus"}}}
{"type":"event_msg","timestamp":"2026-06-29T02:24:02.237Z","payload":{"type":"token_count","info":null,"rate_limits":{"primary":{"used_percent":100.0,"window_minutes":300,"resets_at":1782709162},"secondary":{"used_percent":16.0,"window_minutes":10080,"resets_at":1783295962},"credits":{"has_credits":true,"unlimited":false,"balance":"336.2474587500"},"plan_type":"plus"}}}
"#,
            "null-info",
        );

        let structured = build_structured(&raw, None, true, true, None);

        assert_eq!(raw.totals.total_tokens, 15);
        assert_eq!(structured.account.plan.as_deref(), Some("plus"));
        assert_eq!(structured.account.credits_remaining, Some(336.2474587500));

        let primary = structured
            .limits
            .iter()
            .find(|limit| limit.name == "primary")
            .expect("primary limit");
        assert_eq!(primary.used_percent, Some(100.0));
        assert_eq!(primary.remaining_percent, Some(0.0));
    }

    #[test]
    fn unlimited_credits_adds_diagnostic_and_null_remaining() {
        let raw = CodexLocalRaw {
            root: "/tmp/.codex".to_string(),
            token_events: 1,
            latest_rate_limits: Some(CodexLocalRateLimits {
                credits_unlimited: true,
                plan_type: Some("pro".to_string()),
                ..CodexLocalRateLimits::default()
            }),
            ..CodexLocalRaw::default()
        };

        let structured = build_structured(&raw, None, true, true, None);

        assert_eq!(structured.account.credits_remaining, None);
        assert!(structured
            .diagnostics
            .contains(&"credits: unlimited".to_string()));
    }

    #[test]
    fn parses_unlimited_credits_object() {
        assert_eq!(
            parse_credits(Some(&serde_json::json!({
                "has_credits": true,
                "unlimited": true
            }))),
            (None, true)
        );
        assert_eq!(
            parse_credits(Some(&serde_json::json!({
                "has_credits": true,
                "unlimited": false,
                "balance": "336.2474587500"
            }))),
            (Some(336.2474587500), false)
        );
    }

    #[test]
    fn formats_unix_seconds_as_utc_rfc3339() {
        assert_eq!(format_unix_utc(1_782_709_162), "2026-06-29T04:59:22Z");
        assert_eq!(format_unix_utc(1_750_003_600), "2025-06-15T16:06:40Z");
    }

    #[test]
    fn decode_raw_parses_serialized_usage() {
        let raw = scan_fixture(
            r#"{"type":"event_msg","timestamp":"2026-06-28T11:00:00Z","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":20,"cached_input_tokens":4,"output_tokens":6,"reasoning_output_tokens":2,"total_tokens":32}},"rate_limits":{"primary":{"used_percent":45,"window_minutes":300,"resets_at":1750003600},"plan_type":"pro"}}}
"#,
            "decode-raw",
        );

        let json = serde_json::to_string(&raw).expect("serialize raw");
        assert_eq!(decode_raw(Some(&json)), Some(raw));
    }

    #[test]
    fn collect_returns_source_data_with_json_raw() {
        let raw = scan_fixture(
            r#"{"type":"event_msg","timestamp":"2026-06-28T11:00:00Z","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":20,"cached_input_tokens":4,"output_tokens":6,"reasoning_output_tokens":2,"total_tokens":32}},"rate_limits":{"primary":{"used_percent":45,"window_minutes":300,"resets_at":1750003600},"plan_type":"pro"}}}
"#,
            "collect-source-data",
        );
        let structured = build_structured(&raw, None, true, true, None);
        let data = SourceData {
            raw: serde_json::to_string(&raw).ok(),
            structured,
            stderr: String::new(),
        };

        assert_eq!(decode_raw(data.raw.as_deref()), Some(raw));
        assert!(data.structured.status.data_available);
    }

    #[test]
    fn raw_serializes_to_json() {
        let raw = scan_fixture(
            r#"{"type":"event_msg","timestamp":"2026-06-28T11:00:00Z","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":20,"cached_input_tokens":4,"output_tokens":6,"reasoning_output_tokens":2,"total_tokens":32}},"rate_limits":{"primary":{"used_percent":45,"window_minutes":300,"resets_at":1750003600},"plan_type":"pro"}}}
"#,
            "raw-json",
        );

        let json = serde_json::to_value(&raw).expect("serialize raw");
        assert_eq!(json["token_events"], 1);
        assert_eq!(json["totals"]["total_tokens"], 32);
        assert_eq!(json["latest_rate_limits"]["plan_type"], "pro");
    }
}
