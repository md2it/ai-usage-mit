use std::io;

use chrono::Utc;

use crate::infra::process::run_provider;
use crate::types::{
    LimitInfo, MoneyUsage, ProviderRun, SourceData, SourceStatus, StructuredSourceInfo, TokenUsage,
    UsageInfo,
};

const CLAUDE_COMMAND: &str = "claude";
const PROVIDER: &str = "claude";
const SOURCE: &str = "claude_cli";
const SOURCE_LINK: &str = "docs/get-info";

pub fn get_usage() -> io::Result<SourceData> {
    let run = capture_provider_run()?;
    Ok(build_source_data(&run))
}

pub fn collect_usage() -> io::Result<SourceData> {
    get_usage()
}

fn capture_provider_run() -> io::Result<ProviderRun> {
    run_provider(&expect_script())
}

pub fn build_source_data(run: &ProviderRun) -> SourceData {
    let mut structured = structured_from_output(&run.compacted_stdout);
    if !run.stderr.trim().is_empty() {
        structured
            .diagnostics
            .push(format!("stderr: {}", run.stderr.trim()));
    }

    SourceData {
        raw: Some(run.compacted_stdout.clone()),
        structured,
        stderr: run.stderr.clone(),
    }
}

pub fn structured_from_output(stdout: &str) -> StructuredSourceInfo {
    let collected_at = utc_now();
    let parsed = parse_claude_cli_output(stdout);
    let raw_data_available = !stdout.is_empty();

    let (status, limits, usage, diagnostics) = if parsed.setup_required {
        (
            SourceStatus {
                data_available: false,
                access_available: true,
                message: Some("Claude CLI is not ready: interactive setup is required".to_string()),
            },
            Vec::new(),
            UsageInfo::default(),
            Vec::new(),
        )
    } else if parsed.has_usage_data() {
        (
            SourceStatus {
                data_available: true,
                access_available: true,
                message: None,
            },
            parsed.limits,
            parsed.usage,
            parsed.diagnostics,
        )
    } else {
        (
            SourceStatus {
                data_available: false,
                access_available: true,
                message: Some("usage data not found in CLI output".to_string()),
            },
            Vec::new(),
            UsageInfo::default(),
            parsed.diagnostics,
        )
    };
    let data_as_of = live_snapshot_data_as_of(&collected_at, status.data_available);

    StructuredSourceInfo {
        provider: PROVIDER.to_string(),
        source: SOURCE.to_string(),
        source_link: SOURCE_LINK.to_string(),
        status,
        raw_data_available,
        collected_at: Some(collected_at),
        data_as_of,
        account: Default::default(),
        limits,
        usage,
        diagnostics,
    }
}

fn utc_now() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn live_snapshot_data_as_of(collected_at: &str, data_available: bool) -> Option<String> {
    data_available.then(|| collected_at.to_string())
}

struct ParsedClaudeCliOutput {
    limits: Vec<LimitInfo>,
    usage: UsageInfo,
    setup_required: bool,
    diagnostics: Vec<String>,
}

impl ParsedClaudeCliOutput {
    fn has_usage_data(&self) -> bool {
        !self.limits.is_empty()
            || self.usage.money.used_amount.is_some()
            || self.usage.tokens.input.is_some()
            || self.usage.tokens.output.is_some()
            || self.usage.tokens.cache_read.is_some()
            || self.usage.tokens.cache_write.is_some()
    }
}

fn parse_claude_cli_output(input: &str) -> ParsedClaudeCliOutput {
    let lines = normalize_lines(input);

    if lines.iter().any(|line| {
        let compact = compact_for_matching(line);
        compact.contains("selectloginmethod") || compact.contains("choosethetextstyle")
    }) {
        return ParsedClaudeCliOutput {
            limits: Vec::new(),
            usage: UsageInfo::default(),
            setup_required: true,
            diagnostics: Vec::new(),
        };
    }

    let mut limits = Vec::new();
    if let Some(limit) = structured_limit_block(&lines, "Current session", Some(300)) {
        limits.push(limit);
    }
    if let Some(limit) = structured_limit_block(&lines, "Current week", Some(10080)) {
        limits.push(limit);
    }

    let mut usage = UsageInfo::default();
    if let Some(line) = find_line_by_compact_prefix(&lines, "totalcost") {
        usage.money = parse_money_line(&line);
    }
    if let Some(line) = find_line_by_compact_prefix(&lines, "usage") {
        usage.tokens = parse_token_usage_line(&line);
    }

    ParsedClaudeCliOutput {
        limits,
        usage,
        setup_required: false,
        diagnostics: Vec::new(),
    }
}

fn structured_limit_block(
    lines: &[String],
    label: &str,
    window_minutes: Option<u64>,
) -> Option<LimitInfo> {
    let label_compact = compact_for_matching(label);
    let start = lines
        .iter()
        .position(|line| compact_for_matching(line).starts_with(&label_compact))?;

    let used_percent = lines
        .iter()
        .skip(start + 1)
        .take(3)
        .find_map(|line| extract_percent_used(line))
        .and_then(|value| parse_percent_f64(&value));

    let resets_at = lines
        .iter()
        .skip(start + 1)
        .take(5)
        .find_map(|line| extract_resets(line));

    let (used_percent, remaining_percent) = complement_percents(used_percent, None);

    Some(LimitInfo {
        name: label.to_string(),
        window_label: Some(label.to_string()),
        window_minutes,
        resets_at,
        used_percent,
        remaining_percent,
        used_amount: None,
        remaining_amount: None,
        total_amount: None,
        amount_unit: None,
    })
}

fn complement_percents(
    used_percent: Option<f64>,
    remaining_percent: Option<f64>,
) -> (Option<f64>, Option<f64>) {
    match (used_percent, remaining_percent) {
        (Some(used), None) => (Some(used), Some((100.0 - used).max(0.0))),
        (None, Some(remaining)) => (Some((100.0 - remaining).max(0.0)), Some(remaining)),
        (used, remaining) => (used, remaining),
    }
}

fn parse_percent_f64(value: &str) -> Option<f64> {
    value.trim_end_matches('%').parse::<f64>().ok()
}

fn parse_money_line(line: &str) -> MoneyUsage {
    let Some((_, value)) = line.split_once(':') else {
        return MoneyUsage::default();
    };

    let value = value.trim();
    let numeric = value.trim_start_matches('$').trim().parse::<f64>().ok();

    MoneyUsage {
        used_amount: numeric,
        remaining_amount: None,
        total_amount: numeric,
        currency: if value.starts_with('$') {
            Some("usd".to_string())
        } else {
            None
        },
    }
}

fn parse_token_usage_line(line: &str) -> TokenUsage {
    let Some((_, value)) = line.split_once(':') else {
        return TokenUsage::default();
    };

    let mut tokens = TokenUsage::default();

    for segment in value.split(',') {
        let compact = compact_for_matching(segment);
        let amount = extract_leading_number(&compact);

        if compact.contains("input") && !compact.contains("cache") {
            tokens.input = amount;
        } else if compact.contains("output") {
            tokens.output = amount;
        } else if compact.contains("cacheread") {
            tokens.cache_read = amount;
        } else if compact.contains("cachewrite") {
            tokens.cache_write = amount;
        }
    }

    if tokens.input.is_some()
        || tokens.output.is_some()
        || tokens.cache_read.is_some()
        || tokens.cache_write.is_some()
    {
        tokens.total = Some(
            tokens.input.unwrap_or(0)
                + tokens.output.unwrap_or(0)
                + tokens.cache_read.unwrap_or(0)
                + tokens.cache_write.unwrap_or(0),
        );
    }

    tokens
}

fn extract_leading_number(compact_segment: &str) -> Option<u64> {
    let digits = compact_segment
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>();

    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

fn normalize_lines(input: &str) -> Vec<String> {
    input
        .split(['\n', '\r'])
        .map(normalize_terminal_line)
        .filter(|line| !line.is_empty())
        .collect()
}

fn normalize_terminal_line(raw_line: &str) -> String {
    raw_line
        .trim()
        .trim_matches(|character| character == '\u{2502}')
        .trim()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn extract_percent_used(line: &str) -> Option<String> {
    let used_index = line.find("used")?;
    let before_used = &line[..used_index];
    let percent_index = before_used.rfind('%')?;
    let digits = before_used[..percent_index]
        .chars()
        .rev()
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>();

    if digits.is_empty() {
        return None;
    }

    Some(digits.chars().rev().collect::<String>() + "%")
}

fn extract_resets(line: &str) -> Option<String> {
    let compact = compact_for_matching(line);
    if !compact.starts_with("resets") {
        return None;
    }

    line.split_once(' ')
        .map(|(_, value)| value.trim().to_string())
        .or_else(|| {
            line.strip_prefix("Resets")
                .map(|value| value.trim().to_string())
        })
        .filter(|value| !value.is_empty())
}

fn find_line_by_compact_prefix(lines: &[String], prefix: &str) -> Option<String> {
    lines
        .iter()
        .find(|line| compact_for_matching(line).starts_with(prefix))
        .cloned()
}

fn compact_for_matching(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase()
}

fn expect_script() -> String {
    format!(
        r#"set timeout 25
log_user 1
spawn env TERM=xterm-256color COLUMNS=120 LINES=40 sh -c {{stty cols 120 rows 40; exec {CLAUDE_COMMAND} --no-chrome}}
expect {{
    -re {{Choose.*text.*style|Syntax theme}} {{
        send "\r"
        exp_continue
    }}
    -re {{for shortcuts|Do you trust|Select login method}} {{}}
    timeout {{}}
}}
after 500
send -- "/usage\r"
expect {{
    -re {{Usage|Current session|Current week|Resets}} {{}}
    -re {{Select login method|Choose.*text.*style}} {{}}
    timeout {{}}
}}
after 10000
send "\003"
after 500
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
    fn structured_from_representative_cli_output() {
        let structured = structured_from_output(SAMPLE_OUTPUT);

        assert_eq!(structured.provider, "claude");
        assert_eq!(structured.source, "claude_cli");
        assert_eq!(structured.source_link, "docs/get-info");
        assert!(structured.status.access_available);
        assert!(structured.status.data_available);
        assert!(structured.status.message.is_none());
        assert!(structured.raw_data_available);
        assert!(structured.collected_at.is_some());
        assert_eq!(
            structured.data_as_of.as_deref(),
            structured.collected_at.as_deref()
        );

        assert_eq!(structured.limits.len(), 2);

        let session = &structured.limits[0];
        assert_eq!(session.name, "Current session");
        assert_eq!(session.used_percent, Some(40.0));
        assert_eq!(session.remaining_percent, Some(60.0));
        assert_eq!(session.resets_at.as_deref(), Some("2:20am (Asia/Nicosia)"));
        assert_eq!(session.window_minutes, Some(300));

        let week = &structured.limits[1];
        assert_eq!(week.name, "Current week");
        assert_eq!(week.used_percent, Some(73.0));
        assert_eq!(week.remaining_percent, Some(27.0));
        assert_eq!(week.window_minutes, Some(10080));
        assert_eq!(
            week.resets_at.as_deref(),
            Some("Jun 30 at 1pm (Asia/Nicosia)")
        );

        assert_eq!(structured.usage.money.used_amount, Some(0.0));
        assert_eq!(structured.usage.money.currency.as_deref(), Some("usd"));
        assert_eq!(structured.usage.tokens.input, Some(0));
        assert_eq!(structured.usage.tokens.output, Some(0));
        assert_eq!(structured.usage.tokens.cache_read, Some(0));
        assert_eq!(structured.usage.tokens.cache_write, Some(0));
        assert_eq!(structured.usage.tokens.total, Some(0));
    }

    #[test]
    fn structured_marks_interactive_setup_as_unavailable_data() {
        let input = "Select login method\nChoose the text style\n";
        let structured = structured_from_output(input);

        assert!(structured.status.access_available);
        assert!(!structured.status.data_available);
        assert_eq!(
            structured.status.message.as_deref(),
            Some("Claude CLI is not ready: interactive setup is required")
        );
        assert!(structured.limits.is_empty());
    }

    #[test]
    fn structured_marks_missing_usage_data() {
        let structured = structured_from_output("OpenAI Codex\nfor shortcuts");

        assert!(structured.status.access_available);
        assert!(!structured.status.data_available);
        assert_eq!(
            structured.status.message.as_deref(),
            Some("usage data not found in CLI output")
        );
        assert!(structured.raw_data_available);
    }

    #[test]
    fn get_usage_returns_raw_and_structured() {
        let run = ProviderRun {
            compacted_stdout: SAMPLE_OUTPUT.to_string(),
            stderr: String::new(),
        };

        let data = build_source_data(&run);

        assert_eq!(data.raw.as_deref(), Some(SAMPLE_OUTPUT));
        assert!(data.structured.status.data_available);
        assert!(!data.structured.limits.is_empty());
    }

    #[test]
    fn build_source_data_preserves_raw_stdout_and_stderr_diagnostics() {
        let run = ProviderRun {
            compacted_stdout: SAMPLE_OUTPUT.to_string(),
            stderr: "expect warning\n".to_string(),
        };

        let data = build_source_data(&run);

        assert_eq!(data.raw.as_deref(), Some(SAMPLE_OUTPUT));
        assert_eq!(data.stderr, "expect warning\n");
        assert!(data.structured.status.data_available);
        assert!(data
            .structured
            .diagnostics
            .iter()
            .any(|entry| entry.contains("stderr: expect warning")));
    }
}
