use std::io;

use crate::infra::diagnostics::Diagnostics;
use crate::infra::process::run_provider;
use crate::types::ProviderRun;

const CLAUDE_COMMAND: &str = "claude";

pub fn get_usage(diagnostics: &Diagnostics) -> io::Result<ProviderRun> {
    run_provider(
        diagnostics,
        "claude",
        Some("claude"),
        &expect_script(),
        "accept default theme if first-run wizard appears\n/usage\r\nctrl-c twice\n",
    )
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

pub fn extract_usage_summary(input: &str) -> Option<String> {
    let lines = input
        .split(['\n', '\r'])
        .map(normalize_terminal_line)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    if lines.iter().any(|line| {
        let compact = compact_for_matching(line);
        compact.contains("selectloginmethod") || compact.contains("choosethetextstyle")
    }) {
        return Some(
            "Claude usage:\nClaude CLI is not ready: interactive setup is required\n".to_string(),
        );
    }

    let current_session = extract_claude_limit_block(&lines, "Current session");
    let current_week = extract_claude_limit_block(&lines, "Current week");
    let total_cost = find_line_by_compact_prefix(&lines, "totalcost");
    let token_usage = find_line_by_compact_prefix(&lines, "usage");

    if current_session.is_none()
        && current_week.is_none()
        && total_cost.is_none()
        && token_usage.is_none()
    {
        return None;
    }

    let mut summary = String::from("Claude usage:\n");

    if let Some(value) = current_session {
        summary.push_str(&value);
        summary.push('\n');
    }

    if let Some(value) = current_week {
        summary.push_str(&value);
        summary.push('\n');
    }

    if let Some(value) = total_cost.and_then(|line| format_claude_metric_line(&line)) {
        summary.push_str(&value);
        summary.push('\n');
    }

    if let Some(value) = token_usage.and_then(|line| format_claude_metric_line(&line)) {
        summary.push_str(&value);
        summary.push('\n');
    }

    Some(summary)
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

fn extract_claude_limit_block(lines: &[String], label: &str) -> Option<String> {
    let label_compact = compact_for_matching(label);
    let start = lines
        .iter()
        .position(|line| compact_for_matching(line).starts_with(&label_compact))?;
    let percent = lines
        .iter()
        .skip(start + 1)
        .take(3)
        .find_map(|line| extract_percent_used(line))?;
    let resets = lines
        .iter()
        .skip(start + 1)
        .take(5)
        .find_map(|line| extract_resets(line));

    let mut summary = format!("{label}: {percent} used");
    if let Some(resets) = resets {
        summary.push_str(" (resets ");
        summary.push_str(&format_claude_reset(&resets));
        summary.push(')');
    }

    Some(summary)
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

fn format_claude_metric_line(line: &str) -> Option<String> {
    let compact = compact_for_matching(line);

    if compact.starts_with("totalcost") {
        let value = line.split_once(':')?.1.trim();
        return Some(format!("Total cost: {value}"));
    }

    if compact.starts_with("usage") {
        let value = line.split_once(':')?.1.trim();
        return Some(format!("Tokens: {}", readable_compact_usage(value)));
    }

    None
}

fn readable_compact_usage(value: &str) -> String {
    let mut output = String::new();
    let mut previous = None;

    for character in value.chars() {
        if character == ',' {
            output.push_str(", ");
        } else {
            if character.is_ascii_alphabetic()
                && previous
                    .map(|previous: char| previous.is_ascii_digit())
                    .unwrap_or(false)
            {
                output.push(' ');
            }
            output.push(character);
        }
        previous = Some(character);
    }

    output
        .replace("cacheread", "cache read")
        .replace("cachewrite", "cache write")
}

fn format_claude_reset(value: &str) -> String {
    let with_timezone_space = value.replace('(', " (");
    let mut chars = with_timezone_space.chars().peekable();
    let mut month = String::new();

    while chars
        .peek()
        .map(|character| character.is_ascii_alphabetic())
        .unwrap_or(false)
    {
        month.push(chars.next().expect("peeked character exists"));
    }

    let mut day = String::new();
    while chars
        .peek()
        .map(|character| character.is_ascii_digit())
        .unwrap_or(false)
    {
        day.push(chars.next().expect("peeked character exists"));
    }

    let rest = chars.collect::<String>();
    if month.len() == 3 && !day.is_empty() && rest.starts_with("at") {
        return format!("{month} {day} at {}", rest.trim_start_matches("at"));
    }

    with_timezone_space
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
