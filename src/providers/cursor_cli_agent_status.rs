use std::io;

use crate::infra::diagnostics::Diagnostics;
use crate::infra::process::run_provider;
use crate::types::ProviderRun;

const CURSOR_COMMAND: &str = "cursor";

pub fn get_usage(diagnostics: &Diagnostics) -> io::Result<ProviderRun> {
    run_provider(
        diagnostics,
        "cursor",
        Some("cursor"),
        &expect_script(),
        "cursor agent about\ncursor agent status\n",
    )
}

fn expect_script() -> String {
    format!(
        r#"set timeout 25
log_user 1
spawn env TERM=xterm-256color COLUMNS=120 LINES=40 sh -c {{stty cols 120 rows 40; {CURSOR_COMMAND} agent about; printf '\n'; {CURSOR_COMMAND} agent status}}
expect {{
    -re {{Subscription Tier|Logged in|not authenticated|command not found}} {{}}
    timeout {{}}
}}
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
        compact.contains("commandnotfound") || compact.contains("nosuchfileordirectory")
    }) {
        return Some(
            "Cursor usage:\nCursor CLI not found: install the `cursor` shell command\n".to_string(),
        );
    }

    let status = lines
        .iter()
        .find(|line| line.contains("Logged in as") || line.contains("not authenticated"))
        .cloned();
    let model = extract_cursor_about_value(&lines, "Model");
    let subscription_tier = extract_cursor_about_value(&lines, "Subscription Tier");
    let cli_version = extract_cursor_about_value(&lines, "CLI Version");

    if status.is_none() && model.is_none() && subscription_tier.is_none() && cli_version.is_none() {
        return None;
    }

    let mut summary = String::from("Cursor usage:\n");

    if let Some(value) = subscription_tier {
        summary.push_str("Subscription tier: ");
        summary.push_str(&value);
        summary.push('\n');
    }

    if let Some(value) = model {
        summary.push_str("Model: ");
        summary.push_str(&value);
        summary.push('\n');
    }

    if let Some(value) = status {
        summary.push_str("Status: ");
        summary.push_str(value.trim_start_matches('\u{2713}').trim());
        summary.push('\n');
    }

    if let Some(value) = cli_version {
        summary.push_str("CLI version: ");
        summary.push_str(&value);
        summary.push('\n');
    }

    summary.push_str("Usage/limits: numeric usage and remaining limits are not exposed by this Cursor CLI build\n");

    Some(summary)
}

fn extract_cursor_about_value(lines: &[String], label: &str) -> Option<String> {
    let label_compact = compact_for_matching(label);

    lines.iter().find_map(|line| {
        let compact = compact_for_matching(line);
        if !compact.starts_with(&label_compact) {
            return None;
        }

        let value = line
            .strip_prefix(label)
            .map(str::trim)
            .filter(|value| !value.is_empty())?;

        Some(value.to_string())
    })
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

fn compact_for_matching(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase()
}
