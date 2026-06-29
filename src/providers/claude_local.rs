use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde_json::{json, Value};

use crate::types::{
    AccountInfo, ActivityUsage, ModelUsage, MoneyUsage, SourceData, SourceStatus,
    StructuredSourceInfo, TokenUsage, UsageInfo,
};

const PROVIDER: &str = "claude";
const SOURCE: &str = "claude_local";
const SOURCE_LINK: &str = "docs/get-info";

#[derive(Default)]
struct ClaudeLocalUsage {
    files: usize,
    sessions: HashSet<String>,
    turns: usize,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_creation_tokens: u64,
    latest_timestamp: Option<String>,
    models: HashMap<String, u64>,
}

pub fn collect() -> io::Result<SourceData> {
    let candidate_roots = default_roots()?;
    let scanned_roots = candidate_roots
        .iter()
        .filter(|root| root.is_dir())
        .cloned()
        .collect::<Vec<_>>();

    if scanned_roots.is_empty() {
        return Ok(SourceData {
            raw: Some(encode_raw(&candidate_roots, &scanned_roots, None)?),
            structured: structured_no_roots(),
            stderr: String::new(),
        });
    }

    let mut usage = ClaudeLocalUsage::default();

    for root in &scanned_roots {
        scan_root(root, &mut usage)?;
    }

    if usage.turns == 0 {
        return Ok(SourceData {
            raw: Some(encode_raw(&candidate_roots, &scanned_roots, Some(&usage))?),
            structured: structured_no_usage(scanned_roots.len()),
            stderr: String::new(),
        });
    }

    Ok(SourceData {
        raw: Some(encode_raw(&candidate_roots, &scanned_roots, Some(&usage))?),
        structured: structured_from_usage(&usage),
        stderr: String::new(),
    })
}

fn default_roots() -> io::Result<Vec<PathBuf>> {
    let home = env::var_os("HOME").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "HOME is not set; cannot locate Claude local transcript roots",
        )
    })?;
    let home = PathBuf::from(home);

    Ok(vec![
        home.join(".config").join("claude").join("projects"),
        home.join(".claude").join("projects"),
        home.join("Library")
            .join("Developer")
            .join("Xcode")
            .join("CodingAssistant")
            .join("ClaudeAgentConfig")
            .join("projects"),
    ])
}

fn scan_root(root: &Path, usage: &mut ClaudeLocalUsage) -> io::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            scan_root(&path, usage)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension == "jsonl")
        {
            scan_jsonl_file(&path, usage)?;
        }
    }

    Ok(())
}

fn scan_jsonl_file(path: &Path, usage: &mut ClaudeLocalUsage) -> io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut seen_messages = HashMap::<String, TurnUsage>::new();
    let mut turns_without_id = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        let Ok(record) = serde_json::from_str::<Value>(line) else {
            continue;
        };

        let Some(turn) = extract_turn_usage(&record) else {
            continue;
        };

        if let Some(message_id) = turn.message_id.clone().filter(|value| !value.is_empty()) {
            seen_messages.insert(message_id, turn);
        } else {
            turns_without_id.push(turn);
        }
    }

    let turn_count = seen_messages.len() + turns_without_id.len();
    if turn_count > 0 {
        usage.files += 1;
    }

    for turn in seen_messages.into_values().chain(turns_without_id) {
        usage.sessions.insert(turn.session_id);
        usage.turns += 1;
        usage.input_tokens += turn.input_tokens;
        usage.output_tokens += turn.output_tokens;
        usage.cache_read_tokens += turn.cache_read_tokens;
        usage.cache_creation_tokens += turn.cache_creation_tokens;

        if let Some(model) = turn.model.filter(|value| !value.is_empty()) {
            *usage.models.entry(model).or_default() += 1;
        }

        if let Some(timestamp) = turn.timestamp.filter(|value| !value.is_empty()) {
            if usage
                .latest_timestamp
                .as_ref()
                .is_none_or(|current| timestamp > *current)
            {
                usage.latest_timestamp = Some(timestamp);
            }
        }
    }

    Ok(())
}

struct TurnUsage {
    session_id: String,
    timestamp: Option<String>,
    model: Option<String>,
    message_id: Option<String>,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_creation_tokens: u64,
}

fn extract_turn_usage(record: &Value) -> Option<TurnUsage> {
    if record.get("type")?.as_str()? != "assistant" {
        return None;
    }

    let session_id = record.get("sessionId")?.as_str()?.to_string();
    let message = record.get("message")?;
    let usage = message.get("usage")?;
    let input_tokens = number_field(usage, "input_tokens");
    let output_tokens = number_field(usage, "output_tokens");
    let cache_read_tokens = number_field(usage, "cache_read_input_tokens");
    let cache_creation_tokens = number_field(usage, "cache_creation_input_tokens");

    if input_tokens + output_tokens + cache_read_tokens + cache_creation_tokens == 0 {
        return None;
    }

    Some(TurnUsage {
        session_id,
        timestamp: record
            .get("timestamp")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        model: message
            .get("model")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        message_id: message
            .get("id")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        input_tokens,
        output_tokens,
        cache_read_tokens,
        cache_creation_tokens,
    })
}

fn number_field(value: &Value, key: &str) -> u64 {
    value.get(key).and_then(Value::as_u64).unwrap_or(0)
}

fn encode_raw(
    candidate_roots: &[PathBuf],
    scanned_roots: &[PathBuf],
    usage: Option<&ClaudeLocalUsage>,
) -> io::Result<String> {
    let mut payload = json!({
        "candidate_roots": path_strings(candidate_roots),
        "scanned_roots": path_strings(scanned_roots),
    });

    if let Some(usage) = usage {
        let total_tokens = usage.input_tokens
            + usage.output_tokens
            + usage.cache_read_tokens
            + usage.cache_creation_tokens;
        let mut models = usage
            .models
            .iter()
            .map(|(model, count)| (model.clone(), json!(count)))
            .collect::<Vec<_>>();
        models.sort_by(|(left, _), (right, _)| left.cmp(right));

        payload["usage"] = json!({
            "files": usage.files,
            "sessions": usage.sessions.iter().collect::<Vec<_>>(),
            "turns": usage.turns,
            "input_tokens": usage.input_tokens,
            "output_tokens": usage.output_tokens,
            "cache_read_tokens": usage.cache_read_tokens,
            "cache_creation_tokens": usage.cache_creation_tokens,
            "total_tokens": total_tokens,
            "models": Value::Object(models.into_iter().collect()),
            "latest_timestamp": usage.latest_timestamp,
        });
    }

    serde_json::to_string(&payload).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn path_strings(paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect()
}

fn structured_base(
    status: SourceStatus,
    raw_data_available: bool,
    data_as_of: Option<String>,
) -> StructuredSourceInfo {
    StructuredSourceInfo {
        provider: PROVIDER.to_string(),
        source: SOURCE.to_string(),
        source_link: SOURCE_LINK.to_string(),
        status,
        raw_data_available,
        collected_at: Some(Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()),
        data_as_of,
        account: AccountInfo::default(),
        limits: Vec::new(),
        usage: UsageInfo::default(),
        diagnostics: Vec::new(),
    }
}

fn structured_no_roots() -> StructuredSourceInfo {
    structured_base(
        SourceStatus {
            data_available: false,
            access_available: true,
            message: Some("local transcript roots were not found".to_string()),
        },
        true,
        None,
    )
}

fn structured_no_usage(root_count: usize) -> StructuredSourceInfo {
    structured_base(
        SourceStatus {
            data_available: false,
            access_available: true,
            message: Some(format!(
                "no token usage found in {root_count} local transcript root(s)"
            )),
        },
        true,
        None,
    )
}

fn structured_from_usage(usage: &ClaudeLocalUsage) -> StructuredSourceInfo {
    let total_tokens = usage.input_tokens
        + usage.output_tokens
        + usage.cache_read_tokens
        + usage.cache_creation_tokens;
    let mut diagnostics = vec![
        "official remaining limit/reset unavailable in local transcripts".to_string(),
    ];
    let data_as_of = usage.latest_timestamp.clone();
    if data_as_of.is_none() {
        diagnostics.push("latest transcript record timestamp is unavailable".to_string());
    }

    StructuredSourceInfo {
        provider: PROVIDER.to_string(),
        source: SOURCE.to_string(),
        source_link: SOURCE_LINK.to_string(),
        status: SourceStatus {
            data_available: true,
            access_available: true,
            message: None,
        },
        raw_data_available: true,
        collected_at: Some(Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()),
        data_as_of,
        account: AccountInfo::default(),
        limits: Vec::new(),
        usage: UsageInfo {
            tokens: TokenUsage {
                input: Some(usage.input_tokens),
                cached_input: None,
                output: Some(usage.output_tokens),
                reasoning_output: None,
                cache_read: Some(usage.cache_read_tokens),
                cache_write: Some(usage.cache_creation_tokens),
                total: Some(total_tokens),
            },
            money: MoneyUsage::default(),
            activity: ActivityUsage {
                events_count: None,
                files_count: Some(usage.files as u64),
                sessions_count: Some(usage.sessions.len() as u64),
                turns_count: Some(usage.turns as u64),
                latest_activity_at: usage.latest_timestamp.clone(),
            },
            models: ModelUsage {
                top_model: top_model(&usage.models).map(str::to_string),
            },
        },
        diagnostics,
    }
}

fn top_model(models: &HashMap<String, u64>) -> Option<&str> {
    models
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(model, _)| model.as_str())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn sample_usage() -> ClaudeLocalUsage {
        let mut usage = ClaudeLocalUsage::default();
        usage.files = 2;
        usage.sessions.extend(["s1".to_string(), "s2".to_string()]);
        usage.turns = 5;
        usage.input_tokens = 100;
        usage.output_tokens = 40;
        usage.cache_read_tokens = 10;
        usage.cache_creation_tokens = 5;
        usage.latest_timestamp = Some("2026-06-28T10:01:00Z".to_string());
        usage.models.insert("claude-sonnet-4-6".to_string(), 3);
        usage.models.insert("claude-haiku-4-5".to_string(), 2);
        usage
    }

    #[test]
    fn scans_usage_and_deduplicates_streaming_message_records() {
        let path = env::temp_dir().join(format!(
            "ai-usage-claude-local-{}.jsonl",
            std::process::id()
        ));
        fs::write(
            &path,
            r#"{"type":"assistant","sessionId":"s1","timestamp":"2026-06-28T10:00:00Z","message":{"id":"m1","model":"claude-sonnet-4-6","usage":{"input_tokens":10,"output_tokens":5,"cache_read_input_tokens":1,"cache_creation_input_tokens":2}}}
{"type":"assistant","sessionId":"s1","timestamp":"2026-06-28T10:01:00Z","message":{"id":"m1","model":"claude-sonnet-4-6","usage":{"input_tokens":30,"output_tokens":7,"cache_read_input_tokens":3,"cache_creation_input_tokens":4}}}
{"type":"assistant","sessionId":"s2","timestamp":"2026-06-28T10:02:00Z","message":{"model":"claude-haiku-4-5","usage":{"input_tokens":0,"output_tokens":0,"cache_read_input_tokens":0,"cache_creation_input_tokens":0}}}
"#,
        )
        .expect("write fixture");

        let mut usage = ClaudeLocalUsage::default();
        scan_jsonl_file(&path, &mut usage).expect("scan fixture");
        let structured = structured_from_usage(&usage);
        let _ = fs::remove_file(&path);

        assert_eq!(usage.files, 1);
        assert_eq!(usage.sessions.len(), 1);
        assert_eq!(usage.turns, 1);
        assert_eq!(usage.input_tokens, 30);
        assert_eq!(usage.output_tokens, 7);
        assert_eq!(usage.cache_read_tokens, 3);
        assert_eq!(usage.cache_creation_tokens, 4);

        assert!(structured.status.data_available);
        assert!(structured.status.access_available);
        assert_eq!(structured.usage.tokens.input, Some(30));
        assert_eq!(structured.usage.tokens.output, Some(7));
        assert_eq!(structured.usage.tokens.cache_read, Some(3));
        assert_eq!(structured.usage.tokens.cache_write, Some(4));
        assert_eq!(structured.usage.tokens.total, Some(44));
        assert_eq!(structured.usage.activity.turns_count, Some(1));
        assert_eq!(
            structured.usage.activity.latest_activity_at.as_deref(),
            Some("2026-06-28T10:01:00Z")
        );
        assert_eq!(
            structured.usage.models.top_model.as_deref(),
            Some("claude-sonnet-4-6")
        );
    }

    #[test]
    fn builds_structured_data_from_representative_usage_sample() {
        let usage = sample_usage();
        let structured = structured_from_usage(&usage);

        assert_eq!(structured.provider, "claude");
        assert_eq!(structured.source, "claude_local");
        assert_eq!(structured.source_link, "docs/get-info");
        assert!(structured.status.data_available);
        assert!(structured.status.access_available);
        assert!(structured.raw_data_available);
        assert_eq!(structured.usage.tokens.input, Some(100));
        assert_eq!(structured.usage.tokens.output, Some(40));
        assert_eq!(structured.usage.tokens.cache_read, Some(10));
        assert_eq!(structured.usage.tokens.cache_write, Some(5));
        assert_eq!(structured.usage.tokens.total, Some(155));
        assert_eq!(structured.usage.activity.files_count, Some(2));
        assert_eq!(structured.usage.activity.sessions_count, Some(2));
        assert_eq!(structured.usage.activity.turns_count, Some(5));
        assert_eq!(
            structured.usage.models.top_model.as_deref(),
            Some("claude-sonnet-4-6")
        );
        assert_eq!(
            structured.data_as_of.as_deref(),
            Some("2026-06-28T10:01:00Z")
        );
        assert!(structured
            .diagnostics
            .iter()
            .any(|entry| entry.contains("official remaining limit/reset unavailable")));
    }

    #[test]
    fn structured_unavailable_when_transcript_roots_are_missing() {
        let structured = structured_no_roots();

        assert!(!structured.status.data_available);
        assert!(structured.status.access_available);
        assert_eq!(
            structured.status.message.as_deref(),
            Some("local transcript roots were not found")
        );
        assert!(structured.raw_data_available);
        assert!(structured.limits.is_empty());
    }

    #[test]
    fn structured_unavailable_when_no_token_usage_is_found() {
        let structured = structured_no_usage(2);

        assert!(!structured.status.data_available);
        assert!(structured.status.access_available);
        assert_eq!(
            structured.status.message.as_deref(),
            Some("no token usage found in 2 local transcript root(s)")
        );
        assert!(structured.raw_data_available);
    }

    #[test]
    fn raw_payload_contains_scanned_roots_and_extracted_usage() {
        let candidate_roots = vec![PathBuf::from("/tmp/.config/claude/projects")];
        let scanned_roots = candidate_roots.clone();
        let usage = sample_usage();

        let raw = encode_raw(&candidate_roots, &scanned_roots, Some(&usage)).expect("encode raw");
        let payload: Value = serde_json::from_str(&raw).expect("parse raw json");

        assert_eq!(
            payload["candidate_roots"][0].as_str(),
            Some("/tmp/.config/claude/projects")
        );
        assert_eq!(payload["usage"]["turns"].as_u64(), Some(5));
        assert_eq!(payload["usage"]["total_tokens"].as_u64(), Some(155));
        assert_eq!(
            payload["usage"]["latest_timestamp"].as_str(),
            Some("2026-06-28T10:01:00Z")
        );
    }
}
