use std::io;

use crate::providers::{claude_cli_usage, codex_cli_status, cursor_api2_usage};
use crate::types::{CursorRun, GetLimitsReport, Source, SourceReport};

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
        Source::CodexCli => {
            let result = codex_cli_status::get_usage()?;
            let summary = codex_cli_status::extract_usage_summary(&result.compacted_stdout)
                .unwrap_or_else(|| "Codex usage: not found in CLI output".to_string());

            Ok(SourceReport {
                source,
                summary,
                stderr: result.stderr,
            })
        }
        Source::ClaudeCli => {
            let result = claude_cli_usage::get_usage()?;
            let summary = claude_cli_usage::extract_usage_summary(&result.compacted_stdout)
                .unwrap_or_else(|| "Claude usage: not found in CLI output".to_string());

            Ok(SourceReport {
                source,
                summary,
                stderr: result.stderr,
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
