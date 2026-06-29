#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Source {
    CodexLocal,
    CodexCli,
    ClaudeHook,
    ClaudeCli,
    ClaudeLocal,
    CursorApi2,
}

impl Source {
    pub const ALL: [Self; 6] = [
        Self::CodexLocal,
        Self::CodexCli,
        Self::ClaudeHook,
        Self::ClaudeCli,
        Self::ClaudeLocal,
        Self::CursorApi2,
    ];

    pub const DEFAULTS: [Self; 3] = [Self::CodexLocal, Self::ClaudeHook, Self::CursorApi2];

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "codex_local" => Ok(Self::CodexLocal),
            "codex_cli" => Ok(Self::CodexCli),
            "claude_hook" => Ok(Self::ClaudeHook),
            "claude_cli" => Ok(Self::ClaudeCli),
            "claude_local" => Ok(Self::ClaudeLocal),
            "cursor_api2" => Ok(Self::CursorApi2),
            _ => Err(format!(
                "unknown source `{value}`; expected one of: codex_local, codex_cli, claude_hook, claude_cli, claude_local, cursor_api2"
            )),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::CodexLocal => "codex-local",
            Self::CodexCli => "codex-cli",
            Self::ClaudeHook => "claude-hook",
            Self::ClaudeCli => "claude-cli",
            Self::ClaudeLocal => "claude-local",
            Self::CursorApi2 => "cursor-api2",
        }
    }

    pub fn heading(self) -> &'static str {
        match self {
            Self::CodexLocal => "CODEX-LOCAL",
            Self::CodexCli => "CODEX-CLI",
            Self::ClaudeHook => "CLAUDE-HOOK",
            Self::ClaudeCli => "CLAUDE-CLI",
            Self::ClaudeLocal => "CLAUDE-LOCAL",
            Self::CursorApi2 => "CURSOR-API2",
        }
    }
}

pub struct ProviderRun {
    pub compacted_stdout: String,
    pub stderr: String,
}

pub struct CursorRun {
    pub summary: String,
    pub stderr: String,
}

pub struct GetLimitsReport {
    pub summaries: Vec<String>,
    pub stderr: String,
}

pub struct SourceReport {
    pub source: Source,
    pub summary: String,
    pub stderr: String,
}
