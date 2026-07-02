use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime};

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::types::{
    AccountInfo, LimitInfo, SourceData, SourceStatus, StructuredSourceInfo, UsageInfo,
};

const STDIN_HOOK_READ_TIMEOUT: Duration = Duration::from_millis(500);
const CACHE_MAX_AGE: Duration = Duration::from_secs(600);
const PROVIDER: &str = "claude";
const SOURCE: &str = "claude_statusline_rate_limits";
const SOURCE_LINK: &str = "docs/get-info/providers/claude.md";
const CACHE_FILE_NAME: &str = "claude-statusline-rate-limits.json";
const STATUSLINE_NOT_CONFIGURED_MESSAGE: &str = concat!(
    "Claude statusline is not configured yet. ",
    "To enable live Claude Code limits, give Claude Code this setup prompt: ",
    "https://github.com/md2it/ai-limits/blob/main/docs/setup/claude-statusline.md"
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PayloadOrigin {
    Hook,
    Cache(Option<SystemTime>),
}

#[derive(Clone, Default)]
struct RateLimits {
    primary_name: &'static str,
    primary_label: &'static str,
    primary: Option<RateLimitWindow>,
    secondary_name: &'static str,
    secondary_label: &'static str,
    secondary: Option<RateLimitWindow>,
    credits: Option<f64>,
    plan_type: Option<String>,
}

#[derive(Clone, Default)]
struct RateLimitWindow {
    used_percent: Option<f64>,
    window_minutes: Option<u64>,
    resets_at: Option<String>,
}

pub fn collect() -> io::Result<SourceData> {
    if let Some(payload) = read_hook_payload_from_stdin()? {
        if payload_has_supported_rate_limits(&payload) {
            write_cache(&payload)?;
        }
        return Ok(build_source_data_with_origin(payload, PayloadOrigin::Hook));
    }

    if let Some((payload, modified_at)) = read_fresh_cache()? {
        return Ok(build_source_data_with_origin(
            payload,
            PayloadOrigin::Cache(Some(modified_at)),
        ));
    }

    Ok(unavailable_source_data(STATUSLINE_NOT_CONFIGURED_MESSAGE))
}

pub fn structured_from_payload(payload: &str, from_cache: bool) -> StructuredSourceInfo {
    build_structured_from_payload(
        payload,
        if from_cache {
            PayloadOrigin::Cache(None)
        } else {
            PayloadOrigin::Hook
        },
    )
}

pub fn unavailable_structured(message: &str) -> StructuredSourceInfo {
    unavailable_source_data(message).structured
}

pub fn build_source_data(payload: String, from_cache: bool) -> SourceData {
    build_source_data_with_origin(
        payload,
        if from_cache {
            PayloadOrigin::Cache(None)
        } else {
            PayloadOrigin::Hook
        },
    )
}

fn build_source_data_with_origin(payload: String, origin: PayloadOrigin) -> SourceData {
    SourceData {
        raw: Some(payload.clone()),
        structured: build_structured_from_payload(&payload, origin),
        stderr: String::new(),
    }
}

fn build_structured_from_payload(payload: &str, origin: PayloadOrigin) -> StructuredSourceInfo {
    let mut diagnostics = origin_diagnostics(origin);
    let collected_at = utc_now();

    fn finish(
        origin: PayloadOrigin,
        collected_at: String,
        mut diagnostics: Vec<String>,
        status: SourceStatus,
        raw_data_available: bool,
        account: AccountInfo,
        limits: Vec<LimitInfo>,
        usage: UsageInfo,
    ) -> StructuredSourceInfo {
        let (data_as_of, data_as_of_diagnostics) =
            data_as_of_for_payload(origin, &collected_at, status.data_available);
        diagnostics.extend(data_as_of_diagnostics);

        StructuredSourceInfo {
            provider: PROVIDER.to_string(),
            source: SOURCE.to_string(),
            source_link: SOURCE_LINK.to_string(),
            status,
            raw_data_available,
            collected_at: Some(collected_at),
            data_as_of,
            account,
            limits,
            usage,
            diagnostics,
        }
    }

    let payload = payload.trim();

    if payload.is_empty() {
        return finish(
            origin,
            collected_at,
            diagnostics,
            SourceStatus {
                data_available: false,
                access_available: true,
                message: Some("hook stdin payload is empty".to_string()),
            },
            true,
            AccountInfo::default(),
            Vec::new(),
            Default::default(),
        );
    }

    let record = match serde_json::from_str::<Value>(payload) {
        Ok(record) => record,
        Err(_) => {
            diagnostics.push("hook stdin payload is not valid JSON".to_string());
            return finish(
                origin,
                collected_at,
                diagnostics,
                SourceStatus {
                    data_available: false,
                    access_available: true,
                    message: Some("hook stdin payload is not valid JSON".to_string()),
                },
                true,
                AccountInfo::default(),
                Vec::new(),
                Default::default(),
            );
        }
    };

    let Some(rate_limits_value) = locate_rate_limits(&record) else {
        diagnostics.push("`rate_limits` field is missing in hook payload".to_string());
        return finish(
            origin,
            collected_at,
            diagnostics,
            SourceStatus {
                data_available: false,
                access_available: true,
                message: Some("`rate_limits` field is missing in hook payload".to_string()),
            },
            true,
            AccountInfo::default(),
            Vec::new(),
            Default::default(),
        );
    };

    let rate_limits = parse_rate_limits(rate_limits_value);
    let limits = build_structured_limits(&rate_limits);
    if limits.is_empty() && rate_limits.credits.is_none() && rate_limits.plan_type.is_none() {
        diagnostics.push("`rate_limits` has no supported limit fields".to_string());
        return finish(
            origin,
            collected_at,
            diagnostics,
            SourceStatus {
                data_available: false,
                access_available: true,
                message: Some("`rate_limits` has no supported limit fields".to_string()),
            },
            true,
            account_from_rate_limits(&rate_limits),
            limits,
            Default::default(),
        );
    }

    finish(
        origin,
        collected_at,
        diagnostics,
        SourceStatus {
            data_available: true,
            access_available: true,
            message: None,
        },
        true,
        account_from_rate_limits(&rate_limits),
        limits,
        Default::default(),
    )
}

fn utc_now() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn data_as_of_for_payload(
    origin: PayloadOrigin,
    collected_at: &str,
    data_available: bool,
) -> (Option<String>, Vec<String>) {
    if !data_available {
        return (None, Vec::new());
    }

    match origin {
        PayloadOrigin::Hook => (Some(collected_at.to_string()), Vec::new()),
        PayloadOrigin::Cache(modified_at) => match modified_at.and_then(system_time_to_rfc3339) {
            Some(timestamp) => (Some(timestamp), Vec::new()),
            None => (
                None,
                vec!["data_as_of unavailable: cache modified time is unknown".to_string()],
            ),
        },
    }
}

fn system_time_to_rfc3339(time: SystemTime) -> Option<String> {
    time.duration_since(SystemTime::UNIX_EPOCH)
        .ok()
        .and_then(|duration| {
            DateTime::<Utc>::from_timestamp(duration.as_secs() as i64, duration.subsec_nanos())
                .map(|value| value.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        })
}

fn unavailable_source_data(message: &str) -> SourceData {
    SourceData {
        raw: None,
        structured: StructuredSourceInfo {
            provider: PROVIDER.to_string(),
            source: SOURCE.to_string(),
            source_link: SOURCE_LINK.to_string(),
            status: SourceStatus {
                data_available: false,
                access_available: false,
                message: Some(message.to_string()),
            },
            raw_data_available: false,
            collected_at: None,
            data_as_of: None,
            account: AccountInfo::default(),
            limits: Vec::new(),
            usage: Default::default(),
            diagnostics: Vec::new(),
        },
        stderr: String::new(),
    }
}

fn origin_diagnostics(origin: PayloadOrigin) -> Vec<String> {
    match origin {
        PayloadOrigin::Hook => vec!["data origin: hook stdin".to_string()],
        PayloadOrigin::Cache(_) => {
            vec![format!(
                "data origin: cache (~/.config/ai-limits/{CACHE_FILE_NAME})"
            )]
        }
    }
}

fn account_from_rate_limits(rate_limits: &RateLimits) -> AccountInfo {
    AccountInfo {
        plan: rate_limits.plan_type.clone(),
        credits_total: None,
        credits_used: None,
        credits_remaining: rate_limits.credits,
    }
}

fn build_structured_limits(rate_limits: &RateLimits) -> Vec<LimitInfo> {
    let mut limits = Vec::new();
    if let Some(limit) = structured_limit(
        rate_limits.primary_name,
        rate_limits.primary_label,
        &rate_limits.primary,
    ) {
        limits.push(limit);
    }
    if let Some(limit) = structured_limit(
        rate_limits.secondary_name,
        rate_limits.secondary_label,
        &rate_limits.secondary,
    ) {
        limits.push(limit);
    }
    limits
}

fn structured_limit(
    name: &str,
    window_label: &str,
    window: &Option<RateLimitWindow>,
) -> Option<LimitInfo> {
    let window = window.as_ref()?;
    let used_percent = window.used_percent;
    Some(LimitInfo {
        name: name.to_string(),
        window_label: Some(window_label.to_string()),
        window_minutes: window.window_minutes,
        resets_at: window.resets_at.clone(),
        used_percent,
        remaining_percent: used_percent.map(remaining_percent),
        used_amount: None,
        remaining_amount: None,
        total_amount: None,
        amount_unit: None,
    })
}

fn remaining_percent(used_percent: f64) -> f64 {
    let raw = (100.0 - used_percent).max(0.0);
    (raw * 10.0).round() / 10.0
}

fn read_hook_payload_from_stdin() -> io::Result<Option<String>> {
    if io::stdin().is_terminal() {
        return Ok(None);
    }

    let payload = read_stdin_with_timeout(STDIN_HOOK_READ_TIMEOUT)?;
    if payload.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(payload))
    }
}

fn read_stdin_with_timeout(timeout: Duration) -> io::Result<String> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut payload = String::new();
        let result = io::stdin().read_to_string(&mut payload).map(|_| payload);
        let _ = sender.send(result);
    });

    match receiver.recv_timeout(timeout) {
        Ok(Ok(payload)) => Ok(payload),
        Ok(Err(error)) => Err(error),
        Err(mpsc::RecvTimeoutError::Timeout) | Err(mpsc::RecvTimeoutError::Disconnected) => {
            Ok(String::new())
        }
    }
}

fn cache_path() -> io::Result<PathBuf> {
    let home = std::env::var_os("HOME")
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("ai-limits")
        .join(CACHE_FILE_NAME))
}

fn write_cache(payload: &str) -> io::Result<()> {
    let path = cache_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, payload)
}

fn read_fresh_cache() -> io::Result<Option<(String, SystemTime)>> {
    let path = cache_path()?;
    let metadata = match fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };

    let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
    if modified.elapsed().unwrap_or(CACHE_MAX_AGE) > CACHE_MAX_AGE {
        return Ok(None);
    }

    let payload = fs::read_to_string(&path)?;
    if payload.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some((payload, modified)))
    }
}

fn locate_rate_limits(record: &Value) -> Option<&Value> {
    record
        .get("rate_limits")
        .or_else(|| record.pointer("/payload/rate_limits"))
}

fn payload_has_supported_rate_limits(payload: &str) -> bool {
    let Ok(record) = serde_json::from_str::<Value>(payload.trim()) else {
        return false;
    };
    let Some(rate_limits_value) = locate_rate_limits(&record) else {
        return false;
    };

    let rate_limits = parse_rate_limits(rate_limits_value);
    !build_structured_limits(&rate_limits).is_empty()
        || rate_limits.credits.is_some()
        || rate_limits.plan_type.is_some()
}

fn parse_rate_limits(value: &Value) -> RateLimits {
    let (primary_name, primary_label, primary) = if value.get("five_hour").is_some() {
        (
            "five_hour",
            "5-hour window",
            parse_named_window(value.get("five_hour"), 300),
        )
    } else {
        (
            "primary",
            "Primary window",
            parse_rate_limit_window(value.get("primary")),
        )
    };

    let (secondary_name, secondary_label, secondary) = if value.get("seven_day").is_some() {
        (
            "seven_day",
            "7-day window",
            parse_named_window(value.get("seven_day"), 10080),
        )
    } else {
        (
            "secondary",
            "Secondary window",
            parse_rate_limit_window(value.get("secondary")),
        )
    };

    RateLimits {
        primary_name,
        primary_label,
        primary,
        secondary_name,
        secondary_label,
        secondary,
        credits: value.get("credits").and_then(number_f64_any),
        plan_type: value
            .get("plan_type")
            .and_then(Value::as_str)
            .map(ToString::to_string),
    }
}

fn parse_named_window(value: Option<&Value>, default_minutes: u64) -> Option<RateLimitWindow> {
    let mut window = parse_rate_limit_window(value)?;
    if window.window_minutes.is_none() {
        window.window_minutes = Some(default_minutes);
    }
    Some(window)
}

fn parse_rate_limit_window(value: Option<&Value>) -> Option<RateLimitWindow> {
    let value = value?;
    let used_percent = value
        .get("used_percent")
        .or_else(|| value.get("used_percentage"))
        .and_then(number_f64_any)
        .map(normalize_percent);
    let window_minutes = value.get("window_minutes").and_then(number_u64_any);
    let resets_at = value.get("resets_at").and_then(normalize_reset_at);

    if used_percent.is_none() && window_minutes.is_none() && resets_at.is_none() {
        return None;
    }

    Some(RateLimitWindow {
        used_percent,
        window_minutes,
        resets_at,
    })
}

fn normalize_percent(value: f64) -> f64 {
    (value.clamp(0.0, 100.0) * 10.0).round() / 10.0
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

fn normalize_reset_at(value: &Value) -> Option<String> {
    if let Some(seconds) = number_u64_any(value) {
        return Some(format_unix_utc(seconds));
    }

    let raw = value.as_str()?.trim();
    if raw.is_empty() {
        return None;
    }

    DateTime::parse_from_rfc3339(raw)
        .map(|timestamp| {
            timestamp
                .with_timezone(&Utc)
                .format("%Y-%m-%dT%H:%M:%SZ")
                .to_string()
        })
        .ok()
}

fn format_unix_utc(seconds: u64) -> String {
    DateTime::<Utc>::from_timestamp(seconds as i64, 0)
        .map(|value| value.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        .unwrap_or_else(|| format!("{seconds} (unix)"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_PAYLOAD: &str = r#"{
  "rate_limits": {
    "primary": {"used_percent":"45","window_minutes":"300","resets_at":"1750003600"},
    "secondary": {"used_percent":71.9,"window_minutes":10080,"resets_at":1750600000},
    "credits":123.6,
    "plan_type":"max"
  }
}"#;

    const STATUSLINE_PAYLOAD: &str = r#"{
  "rate_limits": {
    "five_hour": {"used_percentage": 1, "resets_at": 1782721800},
    "seven_day": {"used_percentage": 69, "resets_at": 1782813600}
  }
}"#;

    #[test]
    fn structured_data_from_representative_hook_payload() {
        let data = build_source_data(SAMPLE_PAYLOAD.to_string(), false);
        let structured = &data.structured;

        assert_eq!(data.raw.as_deref(), Some(SAMPLE_PAYLOAD));
        assert_eq!(structured.provider, "claude");
        assert_eq!(structured.source, "claude_statusline_rate_limits");
        assert_eq!(structured.source_link, SOURCE_LINK);
        assert!(structured.status.data_available);
        assert!(structured.status.access_available);
        assert!(structured.status.message.is_none());
        assert!(structured.raw_data_available);
        assert!(structured.collected_at.is_some());
        assert_eq!(
            structured.data_as_of.as_deref(),
            structured.collected_at.as_deref()
        );
        assert_eq!(structured.account.plan.as_deref(), Some("max"));
        assert_eq!(structured.account.credits_remaining, Some(123.6));
        assert_eq!(structured.limits.len(), 2);

        let primary = &structured.limits[0];
        assert_eq!(primary.name, "primary");
        assert_eq!(primary.window_label.as_deref(), Some("Primary window"));
        assert_eq!(primary.window_minutes, Some(300));
        assert_eq!(primary.used_percent, Some(45.0));
        assert_eq!(primary.remaining_percent, Some(55.0));
        assert_eq!(primary.resets_at.as_deref(), Some("2025-06-15T16:06:40Z"));

        let secondary = &structured.limits[1];
        assert_eq!(secondary.name, "secondary");
        assert_eq!(secondary.used_percent, Some(71.9));
        assert_eq!(secondary.remaining_percent, Some(28.1));
        assert_eq!(secondary.resets_at.as_deref(), Some("2025-06-22T13:46:40Z"));
        assert!(structured
            .diagnostics
            .contains(&"data origin: hook stdin".to_string()));
    }

    #[test]
    fn structured_data_from_official_statusline_payload() {
        let structured = structured_from_payload(STATUSLINE_PAYLOAD, false);

        assert!(structured.status.data_available);
        assert_eq!(structured.limits.len(), 2);
        assert_eq!(structured.limits[0].name, "five_hour");
        assert_eq!(
            structured.limits[0].window_label.as_deref(),
            Some("5-hour window")
        );
        assert_eq!(structured.limits[0].window_minutes, Some(300));
        assert_eq!(structured.limits[0].used_percent, Some(1.0));
        assert_eq!(structured.limits[0].remaining_percent, Some(99.0));
        assert_eq!(structured.limits[1].name, "seven_day");
        assert_eq!(structured.limits[1].window_minutes, Some(10080));
        assert_eq!(structured.limits[1].used_percent, Some(69.0));
        assert_eq!(structured.limits[1].remaining_percent, Some(31.0));
    }

    #[test]
    fn normalizes_statusline_percentages_and_unix_resets_for_five_hour_and_seven_day() {
        let payload = r#"{
  "payload": {
    "rate_limits": {
      "five_hour": {"used_percentage": "100.04", "resets_at": "1782721800"},
      "seven_day": {"used_percentage": -0.04, "resets_at": 1782813600}
    }
  }
}"#;

        let structured = structured_from_payload(payload, false);

        assert!(structured.status.data_available);
        assert_eq!(structured.limits.len(), 2);
        assert_eq!(structured.limits[0].name, "five_hour");
        assert_eq!(structured.limits[0].window_minutes, Some(300));
        assert_eq!(structured.limits[0].used_percent, Some(100.0));
        assert_eq!(structured.limits[0].remaining_percent, Some(0.0));
        assert_eq!(
            structured.limits[0].resets_at.as_deref(),
            Some("2026-06-29T08:30:00Z")
        );
        assert_eq!(structured.limits[1].name, "seven_day");
        assert_eq!(structured.limits[1].window_minutes, Some(10080));
        assert_eq!(structured.limits[1].used_percent, Some(0.0));
        assert_eq!(structured.limits[1].remaining_percent, Some(100.0));
        assert_eq!(
            structured.limits[1].resets_at.as_deref(),
            Some("2026-06-30T10:00:00Z")
        );
    }

    #[test]
    fn normalizes_rfc3339_reset_timestamps_to_utc_seconds() {
        let payload = r#"{
  "rate_limits": {
    "five_hour": {
      "used_percentage": 42.24,
      "resets_at": "2026-06-29T11:30:00+03:00"
    }
  }
}"#;

        let structured = structured_from_payload(payload, false);

        assert!(structured.status.data_available);
        assert_eq!(structured.limits.len(), 1);
        assert_eq!(structured.limits[0].used_percent, Some(42.2));
        assert_eq!(
            structured.limits[0].resets_at.as_deref(),
            Some("2026-06-29T08:30:00Z")
        );
    }

    #[test]
    fn only_supported_rate_limit_payloads_are_cacheable() {
        assert!(payload_has_supported_rate_limits(STATUSLINE_PAYLOAD));
        assert!(!payload_has_supported_rate_limits(
            r#"{"rate_limits":{"five_hour":{"foo":"bar"}}}"#
        ));
        assert!(!payload_has_supported_rate_limits(
            r#"{"payload":{"session_id":"abc"}}"#
        ));
        assert!(!payload_has_supported_rate_limits("{invalid"));
    }

    #[test]
    fn structured_data_marks_missing_rate_limits_as_accessible_but_unavailable() {
        let payload = r#"{"hook_event":"statusline","payload":{"session_id":"abc"}}"#;
        let data = build_source_data(payload.to_string(), false);
        let structured = &data.structured;

        assert_eq!(data.raw.as_deref(), Some(payload));
        assert!(!structured.status.data_available);
        assert!(structured.status.access_available);
        assert!(structured.raw_data_available);
        assert_eq!(
            structured.status.message.as_deref(),
            Some("`rate_limits` field is missing in hook payload")
        );
        assert!(structured
            .diagnostics
            .iter()
            .any(|entry| entry.contains("rate_limits")));
    }

    #[test]
    fn structured_data_marks_invalid_json_with_diagnostics() {
        let payload = "{invalid";
        let structured = structured_from_payload(payload, false);

        assert!(!structured.status.data_available);
        assert!(structured.status.access_available);
        assert!(structured.raw_data_available);
        assert_eq!(
            structured.status.message.as_deref(),
            Some("hook stdin payload is not valid JSON")
        );
        assert!(structured
            .diagnostics
            .iter()
            .any(|entry| entry.contains("valid JSON")));
    }

    #[test]
    fn structured_data_marks_no_access_when_hook_context_missing() {
        let data = unavailable_source_data(STATUSLINE_NOT_CONFIGURED_MESSAGE);

        assert!(data.raw.is_none());
        assert!(!data.structured.status.access_available);
        assert!(!data.structured.status.data_available);
        assert!(!data.structured.raw_data_available);
        let message = data
            .structured
            .status
            .message
            .as_deref()
            .expect("message should explain setup");
        assert!(message.contains("Claude statusline is not configured yet"));
        assert!(message.contains(
            "https://github.com/md2it/ai-limits/blob/main/docs/setup/claude-statusline.md"
        ));
    }

    #[test]
    fn structured_data_notes_cache_origin_in_diagnostics() {
        let structured = structured_from_payload(SAMPLE_PAYLOAD, true);

        assert!(structured
            .diagnostics
            .iter()
            .any(|entry| entry.contains("data origin: cache")));
    }

    #[test]
    fn structured_data_marks_unsupported_rate_limit_fields() {
        let payload = r#"{"rate_limits":{"primary":{"foo":"bar"}}}"#;
        let structured = structured_from_payload(payload, false);

        assert!(!structured.status.data_available);
        assert!(structured.status.access_available);
        assert_eq!(
            structured.status.message.as_deref(),
            Some("`rate_limits` has no supported limit fields")
        );
        assert!(structured.limits.is_empty());
    }
}
