#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Source {
    CodexCli,
    ClaudeCli,
    CursorApi2,
}

impl Source {
    pub const ALL: [Self; 3] = [Self::CodexCli, Self::ClaudeCli, Self::CursorApi2];

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "codex_cli" => Ok(Self::CodexCli),
            "claude_cli" => Ok(Self::ClaudeCli),
            "cursor_api2" => Ok(Self::CursorApi2),
            _ => Err(format!(
                "unknown source `{value}`; expected one of: codex_cli, claude_cli, cursor_api2"
            )),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::CodexCli => "codex-cli",
            Self::ClaudeCli => "claude-cli",
            Self::CursorApi2 => "cursor-api2",
        }
    }

    pub fn heading(self) -> &'static str {
        match self {
            Self::CodexCli => "CODEX-CLI",
            Self::ClaudeCli => "CLAUDE-CLI",
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
