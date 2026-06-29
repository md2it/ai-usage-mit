use std::io;

use crate::providers::{
    claude_cli, claude_hook, claude_local, codex_cli, codex_local, cursor_api2,
};
use crate::types::{Source, SourceData, SourceReport};

pub fn get_limits(sources: &[Source]) -> io::Result<Vec<SourceReport>> {
    sources.iter().map(|source| get_source_limits(*source)).collect()
}

pub fn get_source_limits(source: Source) -> io::Result<SourceReport> {
    let data = match source {
        Source::CodexLocal => codex_local::get_usage()?,
        Source::CodexCli => codex_cli::collect_usage()?,
        Source::ClaudeHook => claude_hook::collect()?,
        Source::ClaudeCli => claude_cli::collect_usage()?,
        Source::ClaudeLocal => claude_local::collect()?,
        Source::CursorApi2 => cursor_api2::collect_usage()?,
    };

    Ok(SourceReport { source, data })
}

pub fn get_source_data(source: Source) -> io::Result<SourceData> {
    get_source_limits(source).map(|report| report.data)
}
