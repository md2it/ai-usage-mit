use std::path::PathBuf;

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
    pub diagnostics_dir: PathBuf,
}
