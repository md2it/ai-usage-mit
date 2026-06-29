use std::io::{self, IsTerminal, Read};

use serde_json::Value;

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

pub fn get_usage_summary_from_stdin() -> io::Result<Option<String>> {
    let mut stdin = io::stdin();
    if stdin.is_terminal() {
        return Ok(None);
    }

    let mut payload = String::new();
    stdin.read_to_string(&mut payload)?;

    Ok(Some(extract_usage_summary_from_hook_payload(&payload)))
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
        "Primary window",
        &rate_limits.primary,
    ));
    summary.push_str(&format_rate_limit_window(
        "Secondary window",
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
    RateLimits {
        primary: parse_rate_limit_window(value.get("primary")),
        secondary: parse_rate_limit_window(value.get("secondary")),
        credits: value.get("credits").and_then(number_f64_any),
        plan_type: value
            .get("plan_type")
            .and_then(Value::as_str)
            .map(ToString::to_string),
    }
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
}
