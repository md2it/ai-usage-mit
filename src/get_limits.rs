use std::io;

use crate::infra::diagnostics::Diagnostics;
use crate::providers::{
    claude_cli_usage, codex_cli_status, cursor_api2_usage, cursor_cli_agent_status,
};
use crate::types::{CursorRun, GetLimitsReport};

pub fn get_limits() -> io::Result<GetLimitsReport> {
    let diagnostics = Diagnostics::create()?;
    diagnostics.event("runtime_start")?;

    let codex_result = codex_cli_status::get_usage(&diagnostics)?;
    let codex_summary = codex_cli_status::extract_usage_summary(&codex_result.compacted_stdout)
        .unwrap_or_else(|| "Codex usage: not found in CLI output".to_string());

    let claude_result = claude_cli_usage::get_usage(&diagnostics)?;
    let claude_summary = claude_cli_usage::extract_usage_summary(&claude_result.compacted_stdout)
        .unwrap_or_else(|| "Claude usage: not found in CLI output".to_string());

    let cursor_result = run_cursor_usage(&diagnostics)?;

    diagnostics.event(&format!(
        "runtime_finish diagnostics_dir={}",
        diagnostics.dir().display()
    ))?;

    let mut stderr = String::new();
    stderr.push_str(&codex_result.stderr);
    stderr.push_str(&claude_result.stderr);
    stderr.push_str(&cursor_result.stderr);

    Ok(GetLimitsReport {
        summaries: vec![codex_summary, claude_summary, cursor_result.summary],
        stderr,
        diagnostics_dir: diagnostics.dir().to_path_buf(),
    })
}

fn run_cursor_usage(diagnostics: &Diagnostics) -> io::Result<CursorRun> {
    diagnostics.event("cursor api2_start")?;

    match cursor_api2_usage::get_usage_summary(diagnostics)? {
        cursor_api2_usage::CursorApiUsageResult::Found(summary) => {
            diagnostics.event("cursor api2_success")?;
            Ok(CursorRun {
                summary,
                stderr: String::new(),
            })
        }
        cursor_api2_usage::CursorApiUsageResult::Unavailable(reason) => {
            diagnostics.event(&format!("cursor api2_unavailable reason={reason}"))?;
            let cursor_result = cursor_cli_agent_status::get_usage(diagnostics)?;
            let fallback =
                cursor_cli_agent_status::extract_usage_summary(&cursor_result.compacted_stdout)
                    .unwrap_or_else(|| "Cursor usage:\nCursor usage: not found\n".to_string());
            Ok(CursorRun {
                summary: format!("Cursor usage:\n{reason}\n\n{fallback}"),
                stderr: cursor_result.stderr,
            })
        }
    }
}
