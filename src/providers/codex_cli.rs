use std::io;

use chrono::Utc;

use crate::infra::process::run_provider;
use crate::types::{
    AccountInfo, LimitInfo, SourceData, SourceStatus, StructuredSourceInfo, UsageInfo,
};

const CODEX_COMMAND: &str = "codex";
const PROVIDER: &str = "codex";
const SOURCE: &str = "codex_cli";
const SOURCE_LINK: &str = "docs/get-info/providers/codex.md";

pub fn collect_usage() -> io::Result<SourceData> {
    let run = run_provider(&expect_script())?;
    let raw = run.compacted_stdout;
    let mut structured = build_structured(&raw);

    if !run.stderr.trim().is_empty() {
        structured
            .diagnostics
            .push(format!("stderr: {}", run.stderr.trim()));
    }

    Ok(SourceData {
        raw: Some(raw),
        structured,
        stderr: run.stderr,
    })
}

pub fn build_structured(raw: &str) -> StructuredSourceInfo {
    let mut limits = Vec::new();
    let mut account = AccountInfo::default();
    let mut diagnostics = Vec::new();
    let mut found_data = false;

    for raw_line in raw.lines() {
        let normalized = normalize_line(raw_line);

        if normalized.starts_with("5h limit:") {
            if let Some(limit) = parse_limit_line("5h limit", "5h", 300, &normalized) {
                upsert_limit(&mut limits, limit);
                found_data = true;
            } else {
                diagnostics.push("could not parse 5h limit line".to_string());
            }
        } else if normalized.starts_with("Weekly limit:") {
            if let Some(limit) = parse_limit_line("Weekly limit", "weekly", 10080, &normalized) {
                upsert_limit(&mut limits, limit);
                found_data = true;
            } else {
                diagnostics.push("could not parse weekly limit line".to_string());
            }
        } else if normalized.starts_with("Credits:") {
            match parse_credits_line(&normalized) {
                Some(credits) => {
                    account.credits_remaining = Some(credits);
                    found_data = true;
                }
                None => diagnostics.push("could not parse credits line".to_string()),
            }
        }
    }

    let (data_available, message) = if found_data {
        (true, None)
    } else if raw.trim().is_empty() {
        (false, Some("Codex CLI returned empty output".to_string()))
    } else {
        (
            false,
            Some("supported limit lines not found in Codex CLI output".to_string()),
        )
    };
    let collected_at = Some(Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string());
    let data_as_of = data_available.then(|| collected_at.clone()).flatten();

    StructuredSourceInfo {
        provider: PROVIDER.to_string(),
        source: SOURCE.to_string(),
        source_link: SOURCE_LINK.to_string(),
        status: SourceStatus {
            data_available,
            access_available: true,
            message,
        },
        raw_data_available: !raw.trim().is_empty(),
        collected_at,
        data_as_of,
        account,
        limits,
        usage: UsageInfo::default(),
        diagnostics,
    }
}

fn normalize_line(raw_line: &str) -> String {
    let line = raw_line
        .trim()
        .trim_matches(|character| character == '\u{2502}')
        .trim();
    strip_progress_bar(line)
}

fn parse_limit_line(
    name: &str,
    window_label: &str,
    window_minutes: u64,
    line: &str,
) -> Option<LimitInfo> {
    let remaining_percent = parse_remaining_percent(line)?;
    let used_percent = Some(100.0 - remaining_percent);

    Some(LimitInfo {
        name: name.to_string(),
        window_label: Some(window_label.to_string()),
        window_minutes: Some(window_minutes),
        resets_at: parse_resets_at(line),
        used_percent,
        remaining_percent: Some(remaining_percent),
        used_amount: None,
        remaining_amount: None,
        total_amount: None,
        amount_unit: None,
    })
}

fn parse_remaining_percent(line: &str) -> Option<f64> {
    let marker = "% left";
    let percent_end = line.find(marker)?;
    let before_marker = line[..percent_end].trim();
    let value = before_marker.rsplit(' ').next()?;
    value.parse().ok()
}

fn parse_resets_at(line: &str) -> Option<String> {
    let marker = "(resets ";
    let start = line.find(marker)? + marker.len();
    let rest = &line[start..];
    let end = rest.find(')')?;
    Some(rest[..end].trim().to_string())
}

fn parse_credits_line(line: &str) -> Option<f64> {
    let after_prefix = line.strip_prefix("Credits:")?.trim();
    after_prefix.split_whitespace().next()?.parse().ok()
}

fn upsert_limit(limits: &mut Vec<LimitInfo>, limit: LimitInfo) {
    if let Some(index) = limits
        .iter()
        .position(|existing| existing.name == limit.name)
    {
        limits[index] = limit;
    } else {
        limits.push(limit);
    }
}

fn strip_progress_bar(line: &str) -> String {
    let Some(bracket_start) = line.find('[') else {
        return line.to_string();
    };
    let Some(bracket_end) = line[bracket_start..].find(']') else {
        return line.to_string();
    };

    let prefix = line[..bracket_start].trim_end();
    let rest = line[bracket_start + bracket_end + 1..].trim_start();

    if rest.is_empty() {
        prefix.to_string()
    } else {
        format!("{prefix} {rest}")
    }
}

fn expect_script() -> String {
    format!(
        r#"set timeout 20
log_user 1
spawn env TERM=xterm-256color COLUMNS=120 LINES=40 sh -c {{stty cols 120 rows 40; exec {CODEX_COMMAND} --no-alt-screen}}
expect {{
    -re {{OpenAI Codex}} {{}}
    timeout {{}}
}}
after 2000
send "\033\[200~/status\033\[201~\r"
expect {{
    -re {{Credits:}} {{set have_usage 1}}
    -re {{refresh requested|5h limit:|Weekly limit:}} {{set have_usage 0}}
    timeout {{set have_usage 0}}
}}
if {{$have_usage == 0}} {{
    after 3000
    send "\033\[200~/status\033\[201~\r"
    expect {{
        -re {{Credits:}} {{}}
        timeout {{}}
    }}
}}
after 1000
send "\003"
expect {{
    eof {{}}
    timeout {{exit 0}}
}}
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_OUTPUT: &str = "\
5h limit: [░░░░░░░░░░░░░░░░░░░░] 0% left (resets 07:59)
Weekly limit: [█████████████████░░░] 84% left (resets 02:59 on 6 Jul)
Credits: 335 credits
";

    #[test]
    fn strips_progress_bar_from_limit_lines() {
        assert_eq!(
            strip_progress_bar("5h limit: [░░░░░░░░░░░░░░░░░░░░] 0% left (resets 07:59)"),
            "5h limit: 0% left (resets 07:59)"
        );
        assert_eq!(
            strip_progress_bar(
                "Weekly limit: [█████████████████░░░] 84% left (resets 02:59 on 6 Jul)"
            ),
            "Weekly limit: 84% left (resets 02:59 on 6 Jul)"
        );
    }

    #[test]
    fn leaves_lines_without_progress_bar_unchanged() {
        assert_eq!(
            strip_progress_bar("5h limit: 0% left (resets 07:59)"),
            "5h limit: 0% left (resets 07:59)"
        );
        assert_eq!(
            strip_progress_bar("Credits: 335 credits"),
            "Credits: 335 credits"
        );
    }

    #[test]
    fn build_structured_parses_representative_cli_output() {
        let info = build_structured(SAMPLE_OUTPUT);

        assert_eq!(info.provider, "codex");
        assert_eq!(info.source, "codex_cli");
        assert!(info.status.access_available);
        assert!(info.status.data_available);
        assert!(info.raw_data_available);
        assert!(info.collected_at.is_some());
        assert_eq!(info.data_as_of.as_deref(), info.collected_at.as_deref());
        assert_eq!(info.limits.len(), 2);
        assert_eq!(info.account.credits_remaining, Some(335.0));

        let five_hour = &info.limits[0];
        assert_eq!(five_hour.name, "5h limit");
        assert_eq!(five_hour.window_label.as_deref(), Some("5h"));
        assert_eq!(five_hour.window_minutes, Some(300));
        assert_eq!(five_hour.remaining_percent, Some(0.0));
        assert_eq!(five_hour.used_percent, Some(100.0));
        assert_eq!(five_hour.resets_at.as_deref(), Some("07:59"));

        let weekly = &info.limits[1];
        assert_eq!(weekly.name, "Weekly limit");
        assert_eq!(weekly.window_label.as_deref(), Some("weekly"));
        assert_eq!(weekly.window_minutes, Some(10080));
        assert_eq!(weekly.remaining_percent, Some(84.0));
        assert_eq!(weekly.used_percent, Some(16.0));
        assert_eq!(weekly.resets_at.as_deref(), Some("02:59 on 6 Jul"));
    }

    #[test]
    fn build_structured_reports_missing_data_when_output_has_no_limits() {
        let info = build_structured("OpenAI Codex\n> welcome\n");

        assert!(info.status.access_available);
        assert!(!info.status.data_available);
        assert!(info.raw_data_available);
        assert_eq!(
            info.status.message.as_deref(),
            Some("supported limit lines not found in Codex CLI output")
        );
        assert!(info.limits.is_empty());
        assert!(info.account.credits_remaining.is_none());
        assert!(info.collected_at.is_some());
        assert!(info.data_as_of.is_none());
    }

    #[test]
    fn build_structured_reports_empty_output() {
        let info = build_structured("");

        assert!(info.status.access_available);
        assert!(!info.status.data_available);
        assert!(!info.raw_data_available);
        assert_eq!(
            info.status.message.as_deref(),
            Some("Codex CLI returned empty output")
        );
    }

    #[test]
    fn build_structured_deduplicates_repeated_limit_lines() {
        let output = "\
5h limit: [████░░░░░░░░░░░░░░░░] 35% left (resets 03:48)
Weekly limit: [████████████░░░░░░░░] 59% left (resets 02:59 on 6 Jul)
5h limit: [████░░░░░░░░░░░░░░░░] 35% left (resets 03:48)
Weekly limit: [████████████░░░░░░░░] 59% left (resets 02:59 on 6 Jul)
Credits: 301 credits
";
        let info = build_structured(output);

        assert_eq!(info.limits.len(), 2);
        assert_eq!(info.limits[0].name, "5h limit");
        assert_eq!(info.limits[1].name, "Weekly limit");
        assert_eq!(info.account.credits_remaining, Some(301.0));
    }

    #[test]
    fn build_structured_keeps_latest_duplicate_limit_values() {
        let output = "\
5h limit: 10% left (resets 07:59)
5h limit: 35% left (resets 03:48)
";
        let info = build_structured(output);

        assert_eq!(info.limits.len(), 1);
        assert_eq!(info.limits[0].remaining_percent, Some(35.0));
    }

    #[test]
    fn build_structured_adds_diagnostics_for_unparseable_limit_line() {
        let info = build_structured("5h limit: unavailable\nCredits: 10 credits\n");

        assert!(info.status.data_available);
        assert_eq!(info.limits.len(), 0);
        assert_eq!(info.account.credits_remaining, Some(10.0));
        assert!(info
            .diagnostics
            .iter()
            .any(|entry| entry.contains("5h limit")));
    }
}
