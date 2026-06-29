use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Default)]
struct TokenTotals {
    input_tokens: u64,
    cached_input_tokens: u64,
    output_tokens: u64,
    reasoning_output_tokens: u64,
    total_tokens: u64,
}

#[derive(Default)]
struct CodexLocalUsage {
    files_scanned: u64,
    token_events: u64,
    totals: TokenTotals,
    latest_timestamp: Option<String>,
    latest_rate_limits_timestamp: Option<String>,
    latest_rate_limits: Option<RateLimits>,
}

#[derive(Clone, Default)]
struct RateLimits {
    primary: Option<RateLimitWindow>,
    secondary: Option<RateLimitWindow>,
    credits: Option<f64>,
    plan_type: Option<String>,
}

#[derive(Clone, Default)]
struct RateLimitWindow {
    used_percent: Option<f64>,
    window_minutes: Option<u64>,
    resets_at: Option<u64>,
}

struct TokenEvent {
    timestamp: Option<String>,
    usage: Option<TokenTotals>,
    rate_limits: Option<RateLimits>,
}

pub fn get_usage_summary() -> io::Result<String> {
    let root = codex_home()?;

    if !root.exists() {
        return Ok(format!(
            "Codex local usage:\nnot found: {}\n",
            root.display()
        ));
    }

    let mut usage = CodexLocalUsage::default();
    scan_dir(&root.join("sessions"), &mut usage)?;
    scan_dir(&root.join("archived_sessions"), &mut usage)?;

    Ok(format_summary(&root, &usage))
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

fn parse_token_usage(record: &Value) -> Option<TokenTotals> {
    let usage_value = record
        .get("last_token_usage")
        .or_else(|| record.pointer("/payload/info/last_token_usage"))?;

    Some(TokenTotals {
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

fn parse_rate_limits(record: &Value) -> Option<RateLimits> {
    let value = record
        .get("rate_limits")
        .or_else(|| record.pointer("/payload/rate_limits"))?;

    Some(RateLimits {
        primary: parse_rate_limit_window(value.get("primary")),
        secondary: parse_rate_limit_window(value.get("secondary")),
        credits: value.get("credits").and_then(number_f64_any),
        plan_type: value
            .get("plan_type")
            .and_then(Value::as_str)
            .map(ToString::to_string),
    })
}

fn parse_rate_limit_window(value: Option<&Value>) -> Option<RateLimitWindow> {
    let value = value?;
    let used_percent = value.get("used_percent").and_then(number_f64_any);
    let window_minutes = value.get("window_minutes").and_then(number_u64_any);
    let resets_at = value.get("resets_at").and_then(number_u64_any);

    if used_percent.is_none() && window_minutes.is_none() && resets_at.is_none() {
        return None;
    }

    Some(RateLimitWindow {
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

fn format_summary(root: &Path, usage: &CodexLocalUsage) -> String {
    let mut summary = String::from("Codex local usage:\n");

    if usage.token_events == 0 {
        summary.push_str("token events: not found\n");
        summary.push_str(&format!("files scanned: {}\n", usage.files_scanned));
        summary.push_str(&format!("source: {}\n", root.display()));
        return summary;
    }

    summary.push_str(&format!("token events: {}\n", usage.token_events));
    summary.push_str(&format!("files scanned: {}\n", usage.files_scanned));
    summary.push_str(&format!(
        "tokens: total {}, input {}, cached input {}, output {}, reasoning output {}\n",
        format_number(usage.totals.total_tokens),
        format_number(usage.totals.input_tokens),
        format_number(usage.totals.cached_input_tokens),
        format_number(usage.totals.output_tokens),
        format_number(usage.totals.reasoning_output_tokens)
    ));
    summary.push('\n');

    if let Some(rate_limits) = &usage.latest_rate_limits {
        if let Some(timestamp) = &usage.latest_rate_limits_timestamp {
            summary.push_str(&format!("Latest activity: {timestamp}\n"));
        }
        summary.push_str("limits:\n");
        summary.push_str(&format_rate_limit_window("primary", &rate_limits.primary));
        summary.push_str(&format_rate_limit_window(
            "secondary",
            &rate_limits.secondary,
        ));

        if let Some(credits) = rate_limits.credits {
            summary.push_str(&format!("credits: {}\n", format_decimal(credits)));
        }
        if let Some(plan_type) = &rate_limits.plan_type {
            summary.push_str(&format!("plan: {plan_type}\n"));
        }
    } else {
        if let Some(timestamp) = &usage.latest_timestamp {
            summary.push_str(&format!("Latest activity: {timestamp}\n"));
        }
        summary.push_str("limits/reset: unavailable in local Codex JSONL\n");
    }

    summary.push_str(&format!("source: {}\n", root.display()));

    summary
}

fn format_rate_limit_window(label: &str, value: &Option<RateLimitWindow>) -> String {
    let Some(value) = value else {
        return format!("{label}: unavailable\n");
    };

    let mut details = Vec::new();
    if let Some(used_percent) = value.used_percent {
        details.push(format!("used {}%", format_decimal(used_percent)));
    }
    if let Some(window_minutes) = value.window_minutes {
        details.push(format!("window {}", format_window(window_minutes)));
    }
    if let Some(resets_at) = value.resets_at {
        details.push(format!("resets {}", format_unix_utc(resets_at)));
    }

    if details.is_empty() {
        format!("{label}: unavailable\n")
    } else {
        format!("{label}: {}\n", details.join(", "))
    }
}

fn format_window(minutes: u64) -> String {
    match minutes {
        300 => "5h (300m)".to_string(),
        10080 => "weekly (10080m)".to_string(),
        _ => format!("{minutes}m"),
    }
}

fn format_unix_utc(seconds: u64) -> String {
    DateTime::<Utc>::from_timestamp(seconds as i64, 0)
        .map(|value| value.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        .unwrap_or_else(|| format!("{seconds} (unix)"))
}

fn format_decimal(value: f64) -> String {
    let rounded = (value * 10.0).round() / 10.0;
    if rounded.fract() == 0.0 {
        format!("{rounded:.0}")
    } else {
        format!("{rounded:.1}")
    }
}

fn format_number(value: u64) -> String {
    let digits = value.to_string();
    let mut formatted = String::new();

    for (index, character) in digits.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            formatted.push(',');
        }
        formatted.push(character);
    }

    formatted.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn extracts_usage_and_limits_from_latest_timestamped_event() {
        let path = env::temp_dir().join(format!(
            "ai-usage-codex-local-{}-1.jsonl",
            std::process::id()
        ));
        fs::write(
            &path,
            r#"{"type":"event_msg","timestamp":"2026-06-28T10:00:00Z","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":10,"cached_input_tokens":2,"output_tokens":3,"reasoning_output_tokens":1,"total_tokens":16}},"rate_limits":{"primary":{"used_percent":12.4,"window_minutes":300,"resets_at":1750000000}}}}
{"type":"event_msg","timestamp":"2026-06-28T11:00:00Z","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":20,"cached_input_tokens":4,"output_tokens":6,"reasoning_output_tokens":2,"total_tokens":32}},"rate_limits":{"primary":{"used_percent":"45","window_minutes":"300","resets_at":"1750003600"},"secondary":{"used_percent":71.9,"window_minutes":10080,"resets_at":1750600000},"credits":123.6,"plan_type":"pro"}}}
"#,
        )
        .expect("write fixture");

        let mut usage = CodexLocalUsage::default();
        scan_file(&path, &mut usage).expect("scan fixture");
        let summary = format_summary(Path::new("/tmp/.codex"), &usage);
        let _ = fs::remove_file(&path);

        assert_eq!(usage.token_events, 2);
        assert_eq!(usage.totals.input_tokens, 30);
        assert_eq!(usage.totals.cached_input_tokens, 6);
        assert_eq!(usage.totals.output_tokens, 9);
        assert_eq!(usage.totals.reasoning_output_tokens, 3);
        assert_eq!(usage.totals.total_tokens, 48);
        assert_eq!(
            usage.latest_timestamp.as_deref(),
            Some("2026-06-28T11:00:00Z")
        );

        assert!(summary.contains("limits:"));
        assert!(summary.contains("Latest activity: 2026-06-28T11:00:00Z"));
        assert!(
            summary.contains("primary: used 45%, window 5h (300m), resets 2025-06-15T16:06:40Z")
        );
        assert!(summary.contains(
            "secondary: used 71.9%, window weekly (10080m), resets 2025-06-22T13:46:40Z"
        ));
        assert!(summary.contains("credits: 123.6"));
        assert!(summary.contains("plan: pro"));
    }

    #[test]
    fn keeps_limits_when_latest_event_has_no_rate_limits() {
        let path = env::temp_dir().join(format!(
            "ai-usage-codex-local-{}-2.jsonl",
            std::process::id()
        ));
        fs::write(
            &path,
            r#"{"type":"event_msg","timestamp":"2026-06-28T10:00:00Z","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":10,"cached_input_tokens":0,"output_tokens":5,"reasoning_output_tokens":0,"total_tokens":15}},"rate_limits":{"primary":{"used_percent":10,"window_minutes":300,"resets_at":1750000000}}}}
{"type":"event_msg","timestamp":"2026-06-28T12:00:00Z","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":5,"cached_input_tokens":0,"output_tokens":2,"reasoning_output_tokens":0,"total_tokens":7}}}}
"#,
        )
        .expect("write fixture");

        let mut usage = CodexLocalUsage::default();
        scan_file(&path, &mut usage).expect("scan fixture");
        let summary = format_summary(Path::new("/tmp/.codex"), &usage);
        let _ = fs::remove_file(&path);

        assert_eq!(
            usage.latest_timestamp.as_deref(),
            Some("2026-06-28T12:00:00Z")
        );
        assert!(summary.contains("token events: 2"));
        assert!(summary.contains("Latest activity: 2026-06-28T10:00:00Z"));
        assert!(summary.contains(
            "primary: used 10%, window 5h (300m), resets 2025-06-15T15:06:40Z"
        ));
    }

    #[test]
    fn accepts_rate_limits_only_token_count_with_null_info() {
        let path = env::temp_dir().join(format!(
            "ai-usage-codex-local-{}-3.jsonl",
            std::process::id()
        ));
        fs::write(
            &path,
            r#"{"type":"event_msg","timestamp":"2026-06-29T01:46:39.473Z","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":10,"cached_input_tokens":0,"output_tokens":5,"reasoning_output_tokens":0,"total_tokens":15}},"rate_limits":{"primary":{"used_percent":86.0,"window_minutes":300,"resets_at":1782709162},"plan_type":"plus"}}}
{"type":"event_msg","timestamp":"2026-06-29T02:24:02.237Z","payload":{"type":"token_count","info":null,"rate_limits":{"primary":{"used_percent":100.0,"window_minutes":300,"resets_at":1782709162},"secondary":{"used_percent":16.0,"window_minutes":10080,"resets_at":1783295962},"plan_type":"plus"}}}
"#,
        )
        .expect("write fixture");

        let mut usage = CodexLocalUsage::default();
        scan_file(&path, &mut usage).expect("scan fixture");
        let summary = format_summary(Path::new("/tmp/.codex"), &usage);
        let _ = fs::remove_file(&path);

        assert_eq!(usage.token_events, 2);
        assert_eq!(usage.totals.total_tokens, 15);
        assert_eq!(
            usage.latest_timestamp.as_deref(),
            Some("2026-06-29T02:24:02.237Z")
        );
        assert!(summary.contains("Latest activity: 2026-06-29T02:24:02.237Z"));
        assert!(summary.contains("primary: used 100%, window 5h (300m), resets 2026-06-29T04:59:22Z"));
        assert!(summary.contains(
            "secondary: used 16%, window weekly (10080m), resets 2026-07-05T23:59:22Z"
        ));
    }

    #[test]
    fn formats_unix_seconds_as_utc_rfc3339() {
        assert_eq!(format_unix_utc(1_782_709_162), "2026-06-29T04:59:22Z");
        assert_eq!(format_unix_utc(1_750_003_600), "2025-06-15T16:06:40Z");
    }

    #[test]
    fn formats_large_numbers_with_grouping() {
        assert_eq!(format_number(921394501), "921,394,501");
    }
}
