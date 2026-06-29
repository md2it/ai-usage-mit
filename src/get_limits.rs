use std::io;

use crate::providers::{
    claude_cli, claude_hook, claude_local, codex_cli, codex_local, cursor_api2,
};
use crate::providers::codex_local::CodexLocalRaw;
use crate::types::{
    CursorRun, GetLimitsReport, LimitInfo, Source, SourceReport, StructuredSourceInfo,
};

pub fn get_limits(sources: &[Source]) -> io::Result<GetLimitsReport> {
    let mut summaries = Vec::new();
    let mut stderr = String::new();

    for source in sources {
        let report = get_source_limits(*source)?;
        summaries.push(report.summary);
        stderr.push_str(&report.stderr);
    }

    Ok(GetLimitsReport { summaries, stderr })
}

pub fn get_source_limits(source: Source) -> io::Result<SourceReport> {
    match source {
        Source::CodexLocal => {
            let data = codex_local::get_usage()?;
            let raw = codex_local::decode_raw(data.raw.as_deref())
                .unwrap_or_default();
            Ok(SourceReport {
                source,
                summary: format_codex_local_summary(&raw, &data.structured),
                stderr: data.stderr,
            })
        }
        Source::CodexCli => {
            let result = codex_cli::collect_usage()?;
            let summary = summary_from_codex_structured(&result.structured);

            Ok(SourceReport {
                source,
                summary,
                stderr: result.stderr,
            })
        }
        Source::ClaudeHook => {
            let result = claude_hook::collect()?;
            Ok(SourceReport {
                source,
                summary: format_claude_hook_summary(&result.structured),
                stderr: result.stderr,
            })
        }
        Source::ClaudeCli => {
            let result = claude_cli::collect_usage()?;
            Ok(SourceReport {
                source,
                summary: format_claude_cli_summary(&result.structured),
                stderr: result.stderr,
            })
        }
        Source::ClaudeLocal => {
            let data = claude_local::collect()?;

            Ok(SourceReport {
                source,
                summary: format_claude_local_summary(&data.structured),
                stderr: String::new(),
            })
        }
        Source::CursorApi2 => {
            let result = run_cursor_usage()?;

            Ok(SourceReport {
                source,
                summary: result.summary,
                stderr: result.stderr,
            })
        }
    }
}

fn summary_from_codex_structured(info: &StructuredSourceInfo) -> String {
    let mut summary = String::from("Codex usage:\n");

    if !info.status.data_available {
        summary.push_str(
            info.status
                .message
                .as_deref()
                .unwrap_or("Codex usage unavailable"),
        );
        summary.push('\n');
        return summary;
    }

    for limit in &info.limits {
        summary.push_str(&format_limit_line(limit));
        summary.push('\n');
    }

    if let Some(credits) = info.account.credits_remaining {
        summary.push_str(&format!("Credits: {} credits\n", format_decimal(credits)));
    }

    summary
}

fn format_limit_line(limit: &LimitInfo) -> String {
    let mut line = limit.name.clone();
    line.push(':');

    if let Some(remaining_percent) = limit.remaining_percent {
        line.push_str(&format!(" {}% left", format_decimal(remaining_percent)));
    }

    if let Some(resets_at) = &limit.resets_at {
        line.push_str(&format!(" (resets {resets_at})"));
    }

    line
}

fn format_decimal(value: f64) -> String {
    let rounded = (value * 10.0).round() / 10.0;
    if rounded.fract() == 0.0 {
        format!("{rounded:.0}")
    } else {
        format!("{rounded:.1}")
    }
}

fn format_claude_local_summary(structured: &StructuredSourceInfo) -> String {
    let mut summary = String::from("Claude local usage:\n");

    if !structured.status.data_available {
        if let Some(message) = &structured.status.message {
            summary.push_str(message);
            summary.push('\n');
        }
        return summary;
    }

    summary.push_str("Source: local transcript history\n");
    summary.push_str(&format!(
        "Scope: {} files, {} sessions, {} assistant turns\n",
        structured
            .usage
            .activity
            .files_count
            .unwrap_or_default(),
        structured
            .usage
            .activity
            .sessions_count
            .unwrap_or_default(),
        structured
            .usage
            .activity
            .turns_count
            .unwrap_or_default(),
    ));
    summary.push_str(&format!(
        "Tokens: {} input, {} output, {} cache read, {} cache write, {} total\n",
        format_number(structured.usage.tokens.input.unwrap_or_default()),
        format_number(structured.usage.tokens.output.unwrap_or_default()),
        format_number(structured.usage.tokens.cache_read.unwrap_or_default()),
        format_number(structured.usage.tokens.cache_write.unwrap_or_default()),
        format_number(structured.usage.tokens.total.unwrap_or_default()),
    ));

    if let Some(model) = &structured.usage.models.top_model {
        summary.push_str(&format!("Top model: {model}\n"));
    }

    if let Some(timestamp) = &structured.usage.activity.latest_activity_at {
        summary.push_str(&format!("Latest activity: {timestamp}\n"));
    }

    summary
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

fn format_codex_local_summary(
    raw: &CodexLocalRaw,
    structured: &crate::types::StructuredSourceInfo,
) -> String {
    let mut summary = String::from("Codex local usage:\n");

    if !structured.status.access_available {
        if let Some(message) = &structured.status.message {
            summary.push_str(message);
            summary.push('\n');
        }
        summary.push_str(&format!("source: {}\n", raw.root));
        return summary;
    }

    if !structured.status.data_available {
        if let Some(message) = &structured.status.message {
            summary.push_str(message);
            summary.push('\n');
        }
        summary.push_str(&format!("files scanned: {}\n", raw.files_scanned));
        summary.push_str(&format!("source: {}\n", raw.root));
        return summary;
    }

    summary.push_str(&format!("token events: {}\n", raw.token_events));
    summary.push_str(&format!("files scanned: {}\n", raw.files_scanned));
    summary.push_str(&format!(
        "tokens: total {}, input {}, cached input {}, output {}, reasoning output {}\n",
        format_number(raw.totals.total_tokens),
        format_number(raw.totals.input_tokens),
        format_number(raw.totals.cached_input_tokens),
        format_number(raw.totals.output_tokens),
        format_number(raw.totals.reasoning_output_tokens)
    ));
    summary.push('\n');

    if let Some(rate_limits) = &raw.latest_rate_limits {
        if let Some(timestamp) = &raw.latest_rate_limits_timestamp {
            summary.push_str(&format!("Latest activity: {timestamp}\n"));
        }
        summary.push_str("limits:\n");
        summary.push_str(&format_rate_limit_window("primary", &rate_limits.primary));
        summary.push_str(&format_rate_limit_window(
            "secondary",
            &rate_limits.secondary,
        ));

        if rate_limits.credits_unlimited {
            summary.push_str("credits: unlimited\n");
        } else if let Some(credits) = rate_limits.credits {
            summary.push_str(&format!("credits: {}\n", format_decimal(credits)));
        }
        if let Some(plan_type) = &rate_limits.plan_type {
            summary.push_str(&format!("plan: {plan_type}\n"));
        }
    } else {
        if let Some(timestamp) = &raw.latest_timestamp {
            summary.push_str(&format!("Latest activity: {timestamp}\n"));
        }
        summary.push_str("limits/reset: unavailable in local Codex JSONL\n");
    }

    summary.push_str(&format!("source: {}\n", raw.root));
    summary
}

fn format_rate_limit_window(
    label: &str,
    value: &Option<codex_local::CodexLocalRateLimitWindow>,
) -> String {
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
        details.push(format!(
            "resets {}",
            codex_local_format_unix_utc(resets_at)
        ));
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

fn codex_local_format_unix_utc(seconds: u64) -> String {
    chrono::DateTime::<chrono::Utc>::from_timestamp(seconds as i64, 0)
        .map(|value| value.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        .unwrap_or_else(|| format!("{seconds} (unix)"))
}

fn run_cursor_usage() -> io::Result<CursorRun> {
    let result = cursor_api2::collect_usage()?;
    Ok(CursorRun {
        summary: format_cursor_summary(&result.structured),
        stderr: String::new(),
    })
}

fn format_cursor_summary(structured: &StructuredSourceInfo) -> String {
    let mut summary = String::from("Cursor usage:\n");

    if !structured.status.access_available || !structured.status.data_available {
        if let Some(message) = &structured.status.message {
            summary.push_str(message);
            summary.push('\n');
        }
        return summary;
    }

    if let Some(plan) = structured
        .limits
        .iter()
        .find(|limit| limit.name == "plan_usage")
    {
        if let (Some(used), Some(total)) = (plan.used_amount, plan.total_amount) {
            summary.push_str(&format!(
                "Plan usage: ${used:.2} / ${total:.2}",
                used = used,
                total = total
            ));
            if let Some(percent) = plan.used_percent {
                summary.push_str(&format!(" ({}%)", format_float(percent)));
            }
            summary.push('\n');
            if let Some(remaining) = plan.remaining_amount {
                summary.push_str(&format!("Remaining: ${remaining:.2}\n"));
            }
        } else if let Some(percent) = plan.used_percent {
            summary.push_str(&format!("Plan usage: {}% used\n", format_float(percent)));
        }

        if let Some(label) = &plan.window_label {
            summary.push_str(&format!("Cycle: {label}\n"));
        }
    }

    if let (Some(auto), Some(api)) = (
        structured
            .limits
            .iter()
            .find(|limit| limit.name == "auto")
            .and_then(|limit| limit.used_percent),
        structured
            .limits
            .iter()
            .find(|limit| limit.name == "api_models")
            .and_then(|limit| limit.used_percent),
    ) {
        summary.push_str(&format!(
            "Auto: {}% | API models: {}%\n",
            format_float(auto),
            format_float(api)
        ));
    }

    summary
}

fn format_float(value: f64) -> String {
    if value.fract().abs() < f64::EPSILON {
        format!("{}", value as i64)
    } else {
        format!("{value:.2}")
    }
}

fn format_claude_hook_summary(structured: &StructuredSourceInfo) -> String {
    let mut summary = String::from("Claude usage:\n");

    if !structured.status.access_available || !structured.status.data_available {
        let reason = structured
            .status
            .message
            .as_deref()
            .unwrap_or("Claude hook limits unavailable");
        summary.push_str(&format!("Claude hook live limits unavailable: {reason}\n"));
        summary.push_str("Fallback: Claude CLI /usage or claude_local history\n");
        return summary;
    }

    let source_label = if structured
        .diagnostics
        .iter()
        .any(|entry| entry.contains("data origin: cache"))
    {
        "Claude hook cache"
    } else {
        "Claude hook rate_limits"
    };
    summary.push_str(&format!("Source: {source_label}\n"));

    for limit in &structured.limits {
        summary.push_str(&format_claude_hook_limit_line(limit));
        summary.push('\n');
    }

    if let Some(credits) = structured.account.credits_remaining {
        summary.push_str(&format!("Credits: {}\n", format_decimal(credits)));
    }

    if let Some(plan) = &structured.account.plan {
        summary.push_str(&format!("Plan: {plan}\n"));
    }

    summary
}

fn format_claude_hook_limit_line(limit: &LimitInfo) -> String {
    let label = limit
        .window_label
        .as_deref()
        .unwrap_or(limit.name.as_str());
    let mut details = Vec::new();

    if let Some(used_percent) = limit.used_percent {
        details.push(format!("used {}%", format_decimal(used_percent)));
    }
    if let Some(window_minutes) = limit.window_minutes {
        details.push(format!(
            "window {}",
            format_claude_hook_window(window_minutes)
        ));
    }
    if let Some(resets_at) = &limit.resets_at {
        details.push(format!("resets at {resets_at}"));
    }

    if details.is_empty() {
        format!("{label}: unavailable")
    } else {
        format!("{label}: {}", details.join(", "))
    }
}

fn format_claude_hook_window(minutes: u64) -> String {
    match minutes {
        300 => "5h (300m)".to_string(),
        10080 => "weekly (10080m)".to_string(),
        _ => format!("{minutes}m"),
    }
}

fn format_claude_cli_summary(structured: &StructuredSourceInfo) -> String {
    if !structured.status.data_available {
        if structured
            .status
            .message
            .as_deref()
            .is_some_and(|message| message.contains("interactive setup"))
        {
            return format!(
                "Claude usage:\n{}\n",
                structured.status.message.as_deref().unwrap_or_default()
            );
        }

        return "Claude usage: not found in CLI output".to_string();
    }

    let mut summary = String::from("Claude usage:\n");

    for limit in &structured.limits {
        let label = limit
            .window_label
            .as_deref()
            .unwrap_or(limit.name.as_str());
        let mut line = format!("{label}: ");

        if let Some(used_percent) = limit.used_percent {
            line.push_str(&format!("{}% used", format_decimal(used_percent)));
        } else {
            line.push_str("usage unavailable");
        }

        if let Some(resets_at) = &limit.resets_at {
            line.push_str(&format!(" (resets {resets_at})"));
        }

        summary.push_str(&line);
        summary.push('\n');
    }

    if let Some(amount) = structured.usage.money.used_amount {
        summary.push_str(&format!("Total cost: {}\n", format_claude_cli_money(amount)));
    }

    let tokens = &structured.usage.tokens;
    if tokens.input.is_some()
        || tokens.output.is_some()
        || tokens.cache_read.is_some()
        || tokens.cache_write.is_some()
    {
        let mut parts = Vec::new();
        if let Some(value) = tokens.input {
            parts.push(format!("{value} input"));
        }
        if let Some(value) = tokens.output {
            parts.push(format!("{value} output"));
        }
        if let Some(value) = tokens.cache_read {
            parts.push(format!("{value} cache read"));
        }
        if let Some(value) = tokens.cache_write {
            parts.push(format!("{value} cache write"));
        }
        summary.push_str(&format!("Tokens: {}\n", parts.join(", ")));
    }

    summary
}

fn format_claude_cli_money(amount: f64) -> String {
    format!("${amount:.4}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::{claude_cli, claude_hook};

    const HOOK_SAMPLE_PAYLOAD: &str = r#"{
  "rate_limits": {
    "primary": {"used_percent":"45","window_minutes":"300","resets_at":"1750003600"},
    "secondary": {"used_percent":71.9,"window_minutes":10080,"resets_at":1750600000},
    "credits":123.6,
    "plan_type":"max"
  }
}"#;

    const HOOK_STATUSLINE_PAYLOAD: &str = r#"{
  "rate_limits": {
    "five_hour": {"used_percentage": 1, "resets_at": 1782721800},
    "seven_day": {"used_percentage": 69, "resets_at": 1782813600}
  }
}"#;

    const CLI_SAMPLE_OUTPUT: &str = "\
Current session
40% used
Resets 2:20am (Asia/Nicosia)
Current week
73% used
Resets Jun 30 at 1pm (Asia/Nicosia)
Total cost: $0.0000
Usage: 0input,0output,0cacheread,0cachewrite
";

    #[test]
    fn format_claude_hook_summary_from_representative_payload() {
        let structured =
            claude_hook::structured_from_payload(HOOK_SAMPLE_PAYLOAD, false);
        let summary = format_claude_hook_summary(&structured);

        assert!(summary.contains("Source: Claude hook rate_limits"));
        assert!(summary.contains(
            "Primary window: used 45%, window 5h (300m), resets at 2025-06-15T16:06:40Z"
        ));
        assert!(summary.contains(
            "Secondary window: used 71.9%, window weekly (10080m), resets at 2025-06-22T13:46:40Z"
        ));
        assert!(summary.contains("Credits: 123.6"));
        assert!(summary.contains("Plan: max"));
    }

    #[test]
    fn format_claude_hook_summary_from_statusline_payload() {
        let structured =
            claude_hook::structured_from_payload(HOOK_STATUSLINE_PAYLOAD, false);
        let summary = format_claude_hook_summary(&structured);

        assert!(summary.contains(
            "5-hour window: used 1%, window 5h (300m), resets at 2026-06-29T08:30:00Z"
        ));
        assert!(summary.contains(
            "7-day window: used 69%, window weekly (10080m), resets at 2026-06-30T10:00:00Z"
        ));
    }

    #[test]
    fn format_claude_hook_summary_labels_cache_origin() {
        let structured =
            claude_hook::structured_from_payload(HOOK_STATUSLINE_PAYLOAD, true);
        let summary = format_claude_hook_summary(&structured);

        assert!(summary.contains("Source: Claude hook cache\n"));
        assert!(!summary.contains("Source: Claude hook rate_limits"));
    }

    #[test]
    fn format_claude_hook_summary_handles_missing_rate_limits() {
        let payload = r#"{"hook_event":"statusline","payload":{"session_id":"abc"}}"#;
        let structured = claude_hook::structured_from_payload(payload, false);
        let summary = format_claude_hook_summary(&structured);

        assert!(summary.contains(
            "Claude hook live limits unavailable: `rate_limits` field is missing in hook payload"
        ));
        assert!(summary.contains("Fallback: Claude CLI /usage or claude_local history"));
    }

    #[test]
    fn format_claude_hook_summary_handles_no_access() {
        let structured = claude_hook::unavailable_structured(
            "no hook stdin payload; configure Claude Code statusline to capture payload or use --claude-cli",
        );
        let summary = format_claude_hook_summary(&structured);

        assert!(summary.contains(
            "Claude hook live limits unavailable: no hook stdin payload"
        ));
        assert!(summary.contains("Fallback: Claude CLI /usage or claude_local history"));
    }

    #[test]
    fn format_claude_cli_summary_from_representative_output() {
        let structured = claude_cli::structured_from_output(CLI_SAMPLE_OUTPUT);
        let summary = format_claude_cli_summary(&structured);

        assert!(summary.contains("Current session: 40% used (resets 2:20am (Asia/Nicosia))"));
        assert!(summary.contains("Current week: 73% used (resets Jun 30 at 1pm (Asia/Nicosia))"));
        assert!(summary.contains("Total cost: $0.0000"));
        assert!(summary.contains("Tokens: 0 input, 0 output, 0 cache read, 0 cache write"));
    }

    #[test]
    fn format_claude_cli_summary_handles_interactive_setup() {
        let structured =
            claude_cli::structured_from_output("Select login method\nChoose the text style\n");
        let summary = format_claude_cli_summary(&structured);

        assert_eq!(
            summary,
            "Claude usage:\nClaude CLI is not ready: interactive setup is required\n"
        );
    }

    #[test]
    fn format_claude_cli_summary_handles_missing_usage_data() {
        let structured = claude_cli::structured_from_output("OpenAI Codex\nfor shortcuts");
        let summary = format_claude_cli_summary(&structured);

        assert_eq!(summary, "Claude usage: not found in CLI output");
    }
}
