use std::io;

use crate::infra::process::run_provider;
use crate::types::ProviderRun;

const CODEX_COMMAND: &str = "codex";

pub fn get_usage() -> io::Result<ProviderRun> {
    run_provider(&expect_script())
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

pub fn extract_usage_summary(input: &str) -> Option<String> {
    let mut five_hour_limit = None;
    let mut weekly_limit = None;
    let mut credits = None;

    for raw_line in input.lines() {
        let line = raw_line
            .trim()
            .trim_matches(|character| character == '\u{2502}')
            .trim();
        let normalized = line.split_whitespace().collect::<Vec<_>>().join(" ");

        if normalized.starts_with("5h limit:") {
            five_hour_limit = Some(normalized);
        } else if normalized.starts_with("Weekly limit:") {
            weekly_limit = Some(normalized);
        } else if normalized.starts_with("Credits:") {
            credits = Some(normalized);
        }
    }

    if five_hour_limit.is_none() && weekly_limit.is_none() && credits.is_none() {
        return None;
    }

    let mut summary = String::from("Codex usage:\n");

    if let Some(value) = five_hour_limit {
        summary.push_str(&value);
        summary.push('\n');
    }
    if let Some(value) = weekly_limit {
        summary.push_str(&value);
        summary.push('\n');
    }
    if let Some(value) = credits {
        summary.push_str(&value);
        summary.push('\n');
    }

    Some(summary)
}
