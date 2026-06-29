use std::io;

use crate::providers::{
    claude_cli, claude_hook, claude_local, codex_cli, codex_local, cursor_api2,
};
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
        Source::CodexLocal => Ok(SourceReport {
            source,
            summary: codex_local::get_usage_summary()?,
            stderr: String::new(),
        }),
        Source::CodexCli => {
            let result = codex_cli::get_usage()?;
            let summary = codex_cli::extract_usage_summary(&result.compacted_stdout)
                .unwrap_or_else(|| "Codex usage: not found in CLI output".to_string());

            Ok(SourceReport {
                source,
                summary,
                stderr: result.stderr,
            })
        }
        Source::ClaudeHook => {
            let summary = match claude_hook::get_usage_summary_from_stdin()? {
                Some(summary) => summary,
                None => "Claude usage:\nClaude hook live limits unavailable: stdin hook payload is missing\nFallback: Claude CLI /usage or claude_local history\n".to_string(),
            };

            Ok(SourceReport {
                source,
                summary,
                stderr: String::new(),
            })
        }
        Source::ClaudeCli => {
            let result = claude_cli::get_usage()?;
            let summary = claude_cli::extract_usage_summary(&result.compacted_stdout)
                .unwrap_or_else(|| "Claude usage: not found in CLI output".to_string());

            Ok(SourceReport {
                source,
                summary,
                stderr: result.stderr,
            })
        }
        Source::ClaudeLocal => {
            let summary = match claude_local::get_usage_summary()? {
                claude_local::ClaudeLocalUsageResult::Found(summary) => summary,
                claude_local::ClaudeLocalUsageResult::Unavailable(reason) => reason,
            };

            Ok(SourceReport {
                source,
                summary,
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

fn run_cursor_usage() -> io::Result<CursorRun> {
    match cursor_api2::get_usage_summary()? {
        cursor_api2::CursorApiUsageResult::Found(summary) => Ok(CursorRun {
            summary,
            stderr: String::new(),
        }),
        cursor_api2::CursorApiUsageResult::Unavailable(reason) => Ok(CursorRun {
            summary: format!("Cursor usage:\n{reason}\n"),
            stderr: String::new(),
        }),
    }
}
