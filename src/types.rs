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

    pub const DEFAULTS: [Self; 3] = [Self::CodexLocal, Self::ClaudeLocal, Self::CursorApi2];

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

pub struct SourceReport {
    pub source: Source,
    pub data: SourceData,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct SourceData {
    pub raw: Option<String>,
    pub structured: StructuredSourceInfo,
    pub stderr: String,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct StructuredSourceInfo {
    pub provider: String,
    pub source: String,
    pub source_link: String,
    pub status: SourceStatus,
    pub raw_data_available: bool,
    pub collected_at: Option<String>,
    pub data_as_of: Option<String>,
    pub account: AccountInfo,
    pub limits: Vec<LimitInfo>,
    pub usage: UsageInfo,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct SourceStatus {
    pub data_available: bool,
    pub access_available: bool,
    pub message: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Default, serde::Serialize)]
pub struct AccountInfo {
    pub plan: Option<String>,
    pub credits_total: Option<f64>,
    pub credits_used: Option<f64>,
    pub credits_remaining: Option<f64>,
}

pub type StructuredAccount = AccountInfo;

#[derive(Clone, Debug, PartialEq, Default, serde::Serialize)]
pub struct LimitInfo {
    pub name: String,
    pub window_label: Option<String>,
    pub window_minutes: Option<u64>,
    pub resets_at: Option<String>,
    pub used_percent: Option<f64>,
    pub remaining_percent: Option<f64>,
    pub used_amount: Option<f64>,
    pub remaining_amount: Option<f64>,
    pub total_amount: Option<f64>,
    pub amount_unit: Option<String>,
}

pub type StructuredLimit = LimitInfo;

#[derive(Clone, Debug, PartialEq, Default, serde::Serialize)]
pub struct UsageInfo {
    pub tokens: TokenUsage,
    pub money: MoneyUsage,
    pub activity: ActivityUsage,
    pub models: ModelUsage,
}

#[derive(Clone, Debug, PartialEq, Default, serde::Serialize)]
pub struct TokenUsage {
    pub input: Option<u64>,
    pub cached_input: Option<u64>,
    pub output: Option<u64>,
    pub reasoning_output: Option<u64>,
    pub cache_read: Option<u64>,
    pub cache_write: Option<u64>,
    pub total: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Default, serde::Serialize)]
pub struct MoneyUsage {
    pub used_amount: Option<f64>,
    pub remaining_amount: Option<f64>,
    pub total_amount: Option<f64>,
    pub currency: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Default, serde::Serialize)]
pub struct ActivityUsage {
    pub events_count: Option<u64>,
    pub files_count: Option<u64>,
    pub sessions_count: Option<u64>,
    pub turns_count: Option<u64>,
    pub latest_activity_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Default, serde::Serialize)]
pub struct ModelUsage {
    pub top_model: Option<String>,
}
