use std::io;

use crate::providers::{claude_cli_usage, codex_cli_status, cursor_api2_usage};
use crate::types::{CursorRun, GetLimitsReport};

pub fn get_limits() -> io::Result<GetLimitsReport> {
    let codex_result = codex_cli_status::get_usage()?;
    let codex_summary = codex_cli_status::extract_usage_summary(&codex_result.compacted_stdout)
        .unwrap_or_else(|| "Codex usage: not found in CLI output".to_string());

    let claude_result = claude_cli_usage::get_usage()?;
    let claude_summary = claude_cli_usage::extract_usage_summary(&claude_result.compacted_stdout)
        .unwrap_or_else(|| "Claude usage: not found in CLI output".to_string());

    let cursor_result = run_cursor_usage()?;

    let mut stderr = String::new();
    stderr.push_str(&codex_result.stderr);
    stderr.push_str(&claude_result.stderr);
    stderr.push_str(&cursor_result.stderr);

    Ok(GetLimitsReport {
        summaries: vec![codex_summary, claude_summary, cursor_result.summary],
        stderr,
    })
}

fn run_cursor_usage() -> io::Result<CursorRun> {
    match cursor_api2_usage::get_usage_summary()? {
        cursor_api2_usage::CursorApiUsageResult::Found(summary) => Ok(CursorRun {
            summary,
            stderr: String::new(),
        }),
        cursor_api2_usage::CursorApiUsageResult::Unavailable(reason) => Ok(CursorRun {
            summary: format!("Cursor usage:\n{reason}\n"),
            stderr: String::new(),
        }),
    }
}
