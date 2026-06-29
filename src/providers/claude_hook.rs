use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime};

use serde_json::Value;

const STDIN_HOOK_READ_TIMEOUT: Duration = Duration::from_millis(500);
const CACHE_MAX_AGE: Duration = Duration::from_secs(600);

#[derive(Clone, Default)]
struct RateLimits {
    primary_label: &'static str,
    primary: Option<RateLimitWindow>,
    secondary_label: &'static str,
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

pub fn get_usage_summary() -> io::Result<String> {
    if let Some(payload) = read_hook_payload_from_stdin()? {
        write_cache(&payload)?;
        return Ok(extract_usage_summary_from_hook_payload(&payload));
    }

    if let Some(payload) = read_fresh_cache()? {
        return Ok(with_cache_source_line(&payload));
    }

    Ok(unavailable_summary(
        "no hook stdin payload; configure Claude Code statusline to capture payload or use --claude-cli",
    ))
}

fn with_cache_source_line(payload: &str) -> String {
    let mut summary = extract_usage_summary_from_hook_payload(payload);
    if let Some(rest) = summary.strip_prefix("Claude usage:\nSource: Claude hook rate_limits\n") {
        summary = format!("Claude usage:\nSource: Claude hook cache\n{rest}");
    }
    summary
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
    let home = std::env::var_os("HOME").ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "HOME is not set")
    })?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("ai-usage")
        .join("claude-hook-payload.json"))
}

fn write_cache(payload: &str) -> io::Result<()> {
    let path = cache_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, payload)
}

fn read_fresh_cache() -> io::Result<Option<String>> {
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
        Ok(Some(payload))
    }
}

pub fn extract_usage_summary_from_hook_payload(payload: &str) -> String {
    let payload = payload.trim();
    if payload.is_empty() {
        return unavailable_summary("hook stdin payload is empty");
    }

    let record = match serde_json::from_str::<Value>(payload) {
        Ok(record) => record,
        Err(_) => return unavailable_summary("hook stdin payload is not valid JSON"),
    };

    let Some(rate_limits_value) = locate_rate_limits(&record) else {
        return unavailable_summary("`rate_limits` field is missing in hook payload");
    };

    let rate_limits = parse_rate_limits(rate_limits_value);
    if rate_limits.primary.is_none()
        && rate_limits.secondary.is_none()
        && rate_limits.credits.is_none()
        && rate_limits.plan_type.is_none()
    {
        return unavailable_summary("`rate_limits` has no supported limit fields");
    }

    let mut summary = String::from("Claude usage:\n");
    summary.push_str("Source: Claude hook rate_limits\n");
    summary.push_str(&format_rate_limit_window(
        rate_limits.primary_label,
        &rate_limits.primary,
    ));
    summary.push_str(&format_rate_limit_window(
        rate_limits.secondary_label,
        &rate_limits.secondary,
    ));

    if let Some(credits) = rate_limits.credits {
        summary.push_str(&format!("Credits: {}\n", format_decimal(credits)));
    }

    if let Some(plan_type) = rate_limits.plan_type {
        summary.push_str(&format!("Plan: {plan_type}\n"));
    }

    summary
}

fn locate_rate_limits(record: &Value) -> Option<&Value> {
    record
        .get("rate_limits")
        .or_else(|| record.pointer("/payload/rate_limits"))
}

fn parse_rate_limits(value: &Value) -> RateLimits {
    let (primary_label, primary) = if value.get("five_hour").is_some() {
        (
            "5-hour window",
            parse_named_window(value.get("five_hour"), 300),
        )
    } else {
        (
            "Primary window",
            parse_rate_limit_window(value.get("primary")),
        )
    };

    let (secondary_label, secondary) = if value.get("seven_day").is_some() {
        (
            "7-day window",
            parse_named_window(value.get("seven_day"), 10080),
        )
    } else {
        (
            "Secondary window",
            parse_rate_limit_window(value.get("secondary")),
        )
    };

    RateLimits {
        primary_label,
        primary,
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
        .and_then(number_f64_any);
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
        details.push(format!("resets at {resets_at} (unix)"));
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

fn format_decimal(value: f64) -> String {
    let rounded = (value * 10.0).round() / 10.0;
    if rounded.fract() == 0.0 {
        format!("{rounded:.0}")
    } else {
        format!("{rounded:.1}")
    }
}

fn unavailable_summary(reason: &str) -> String {
    format!(
        "Claude usage:\nClaude hook live limits unavailable: {reason}\nFallback: Claude CLI /usage or claude_local history\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hook_rate_limits_with_supported_fields() {
        let payload = r#"{
  "rate_limits": {
    "primary": {"used_percent":"45","window_minutes":"300","resets_at":"1750003600"},
    "secondary": {"used_percent":71.9,"window_minutes":10080,"resets_at":1750600000},
    "credits":123.6,
    "plan_type":"max"
  }
}"#;

        let summary = extract_usage_summary_from_hook_payload(payload);
        assert!(summary.contains("Source: Claude hook rate_limits"));
        assert!(summary
            .contains("Primary window: used 45%, window 5h (300m), resets at 1750003600 (unix)"));
        assert!(summary.contains(
            "Secondary window: used 71.9%, window weekly (10080m), resets at 1750600000 (unix)"
        ));
        assert!(summary.contains("Credits: 123.6"));
        assert!(summary.contains("Plan: max"));
    }

    #[test]
    fn parses_official_statusline_rate_limits() {
        let payload = r#"{
  "rate_limits": {
    "five_hour": {"used_percentage": 1, "resets_at": 1782721800},
    "seven_day": {"used_percentage": 69, "resets_at": 1782813600}
  }
}"#;

        let summary = extract_usage_summary_from_hook_payload(payload);
        assert!(summary.contains(
            "5-hour window: used 1%, window 5h (300m), resets at 1782721800 (unix)"
        ));
        assert!(summary.contains(
            "7-day window: used 69%, window weekly (10080m), resets at 1782813600 (unix)"
        ));
    }

    #[test]
    fn handles_missing_rate_limits_with_clear_fallback() {
        let payload = r#"{"hook_event":"statusline","payload":{"session_id":"abc"}}"#;
        let summary = extract_usage_summary_from_hook_payload(payload);

        assert!(summary.contains(
            "Claude hook live limits unavailable: `rate_limits` field is missing in hook payload"
        ));
        assert!(summary.contains("Fallback: Claude CLI /usage or claude_local history"));
    }

    #[test]
    fn handles_invalid_json_with_clear_fallback() {
        let summary = extract_usage_summary_from_hook_payload("{invalid");

        assert!(summary
            .contains("Claude hook live limits unavailable: hook stdin payload is not valid JSON"));
        assert!(summary.contains("Fallback: Claude CLI /usage or claude_local history"));
    }

    #[test]
    fn handles_rate_limits_without_supported_fields() {
        let payload = r#"{"rate_limits":{"primary":{"foo":"bar"}}}"#;
        let summary = extract_usage_summary_from_hook_payload(payload);

        assert!(summary.contains(
            "Claude hook live limits unavailable: `rate_limits` has no supported limit fields"
        ));
        assert!(summary.contains("Fallback: Claude CLI /usage or claude_local history"));
    }

    #[test]
    fn with_cache_source_line_relabels_summary() {
        let payload = r#"{"rate_limits":{"five_hour":{"used_percentage":1,"resets_at":1782721800}}}"#;
        let summary = with_cache_source_line(payload);
        assert!(summary.contains("Source: Claude hook cache\n"));
        assert!(!summary.contains("Source: Claude hook rate_limits"));
    }
}
