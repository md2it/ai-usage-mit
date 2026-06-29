use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};

use serde_json::Value;

#[derive(Default)]
pub struct ClaudeLocalUsage {
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

pub enum ClaudeLocalUsageResult {
    Found(String),
    Unavailable(String),
}

pub fn get_usage_summary() -> io::Result<ClaudeLocalUsageResult> {
    let roots = default_roots()?;
    let existing_roots = roots
        .into_iter()
        .filter(|root| root.is_dir())
        .collect::<Vec<_>>();

    if existing_roots.is_empty() {
        return Ok(ClaudeLocalUsageResult::Unavailable(
            "Claude local usage:\nlocal transcript roots were not found\n".to_string(),
        ));
    }

    let mut usage = ClaudeLocalUsage::default();

    for root in &existing_roots {
        scan_root(root, &mut usage)?;
    }

    if usage.turns == 0 {
        return Ok(ClaudeLocalUsageResult::Unavailable(format!(
            "Claude local usage:\nno token usage found in {} local transcript root(s)\n",
            existing_roots.len()
        )));
    }

    Ok(ClaudeLocalUsageResult::Found(format_summary(&usage)))
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

fn format_summary(usage: &ClaudeLocalUsage) -> String {
    let total_tokens = usage.input_tokens
        + usage.output_tokens
        + usage.cache_read_tokens
        + usage.cache_creation_tokens;
    let mut summary = String::from("Claude local usage:\n");
    summary.push_str("Source: local transcript history\n");
    summary.push_str(&format!(
        "Scope: {} files, {} sessions, {} assistant turns\n",
        usage.files,
        usage.sessions.len(),
        usage.turns
    ));
    summary.push_str(&format!(
        "Tokens: {} input, {} output, {} cache read, {} cache write, {} total\n",
        format_number(usage.input_tokens),
        format_number(usage.output_tokens),
        format_number(usage.cache_read_tokens),
        format_number(usage.cache_creation_tokens),
        format_number(total_tokens)
    ));

    if let Some(model) = top_model(&usage.models) {
        summary.push_str(&format!("Top model: {model}\n"));
    }

    if let Some(timestamp) = &usage.latest_timestamp {
        summary.push_str(&format!("Latest activity: {timestamp}\n"));
    }

    summary
}

fn top_model(models: &HashMap<String, u64>) -> Option<&str> {
    models
        .iter()
        .max_by_key(|(_, count)| *count)
        .map(|(model, _)| model.as_str())
}

fn format_number(value: u64) -> String {
    let value = value.to_string();
    let mut output = String::new();

    for (index, character) in value.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            output.push(',');
        }
        output.push(character);
    }

    output.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

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
        let _ = fs::remove_file(&path);

        assert_eq!(usage.files, 1);
        assert_eq!(usage.sessions.len(), 1);
        assert_eq!(usage.turns, 1);
        assert_eq!(usage.input_tokens, 30);
        assert_eq!(usage.output_tokens, 7);
        assert_eq!(usage.cache_read_tokens, 3);
        assert_eq!(usage.cache_creation_tokens, 4);
        assert_eq!(
            usage.latest_timestamp.as_deref(),
            Some("2026-06-28T10:01:00Z")
        );
    }

    #[test]
    fn formats_large_numbers_with_grouping() {
        assert_eq!(format_number(921394501), "921,394,501");
    }
}
