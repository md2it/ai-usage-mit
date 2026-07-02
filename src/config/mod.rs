use std::{env, fs, io, path::PathBuf, time::Duration};

use crate::types::Source;

pub struct Config {
    pub default_sources: Vec<Source>,
    pub watch_interval: Duration,
}

const DEFAULT_CONFIG: &str = "\
default_sources = [
  \"codex_local\",
  \"claude_statusline_rate_limits\",
  \"claude_local\",
  \"cursor_api2\"
]
watch_interval = \"5m\"
";

const DEFAULT_WATCH_INTERVAL: Duration = Duration::from_secs(5 * 60);

pub fn load() -> io::Result<Option<Config>> {
    let path = config_path()?;

    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };

    parse_config(&content).map(Some).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("invalid config {}: {error}", path.display()),
        )
    })
}

pub fn config_path() -> io::Result<PathBuf> {
    let home = env::var_os("HOME").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "HOME is not set; cannot locate ~/.config/ai-limits/config.toml",
        )
    })?;

    Ok(PathBuf::from(home)
        .join(".config")
        .join("ai-limits")
        .join("config.toml"))
}

pub fn write_default(path: &std::path::Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, DEFAULT_CONFIG)
}

fn parse_config(content: &str) -> Result<Config, String> {
    let mut default_sources = None;
    let mut watch_interval = None;
    let mut lines = content.lines();

    while let Some(line) = lines.next() {
        let line = strip_comment(line).trim();

        if line.is_empty() {
            continue;
        }

        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| format!("expected `key = value`, got `{line}`"))?;

        match key.trim() {
            "default_sources" => {
                if default_sources.is_some() {
                    return Err("duplicate `default_sources`".to_string());
                }

                let mut value = value.trim().to_string();

                while value.starts_with('[') && !value.ends_with(']') {
                    let Some(next_line) = lines.next() else {
                        return Err("unterminated `default_sources` array".to_string());
                    };

                    value.push(' ');
                    value.push_str(strip_comment(next_line).trim());
                }

                default_sources = Some(parse_sources_array(&value)?);
            }
            "watch_interval" => {
                if watch_interval.is_some() {
                    return Err("duplicate `watch_interval`".to_string());
                }

                watch_interval = Some(parse_quoted_duration(value.trim())?);
            }
            unknown => return Err(format!("unknown config key `{unknown}`")),
        }
    }

    Ok(Config {
        default_sources: default_sources.unwrap_or_default(),
        watch_interval: watch_interval.unwrap_or(DEFAULT_WATCH_INTERVAL),
    })
}

fn strip_comment(line: &str) -> &str {
    line.split_once('#').map_or(line, |(value, _)| value)
}

fn parse_sources_array(value: &str) -> Result<Vec<Source>, String> {
    let value = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .ok_or_else(|| "expected `default_sources` to be an array".to_string())?;

    let mut sources = Vec::new();

    for item in value.split(',') {
        let item = item.trim();

        if item.is_empty() {
            continue;
        }

        let source = item
            .strip_prefix('"')
            .and_then(|item| item.strip_suffix('"'))
            .ok_or_else(|| format!("expected source name in double quotes, got `{item}`"))?;

        sources.push(Source::parse(source)?);
    }

    Ok(sources)
}

pub fn parse_duration(value: &str) -> Result<Duration, String> {
    let unit = value
        .chars()
        .last()
        .ok_or_else(|| "duration cannot be empty".to_string())?;
    let amount = value
        .strip_suffix(unit)
        .ok_or_else(|| format!("invalid duration `{value}`"))?;
    let amount = amount
        .parse::<u64>()
        .map_err(|_| format!("invalid duration amount `{amount}`"))?;

    if amount == 0 {
        return Err("duration must be greater than zero".to_string());
    }

    let seconds = match unit {
        's' => amount,
        'm' => amount.saturating_mul(60),
        'h' => amount.saturating_mul(60 * 60),
        _ => return Err(format!("unsupported duration unit `{unit}`")),
    };

    Ok(Duration::from_secs(seconds))
}

fn parse_quoted_duration(value: &str) -> Result<Duration, String> {
    let value = value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .ok_or_else(|| format!("expected duration in double quotes, got `{value}`"))?;

    parse_duration(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_config_sources() {
        let parsed = parse_config(DEFAULT_CONFIG).expect("default config should parse");

        assert_eq!(
            parsed.default_sources,
            vec![
                Source::CodexLocal,
                Source::ClaudeStatusline,
                Source::ClaudeLocal,
                Source::CursorApi2
            ]
        );
        assert_eq!(parsed.watch_interval, Duration::from_secs(5 * 60));
    }

    #[test]
    fn parse_config_accepts_claude_statusline() {
        let parsed = parse_config("default_sources = [\"claude_statusline_rate_limits\"]")
            .expect("claude statusline should be accepted");

        assert_eq!(parsed.default_sources, vec![Source::ClaudeStatusline]);
    }

    #[test]
    fn parses_watch_interval() {
        let parsed = parse_config("watch_interval = \"30s\"").expect("watch interval should parse");

        assert_eq!(parsed.watch_interval, Duration::from_secs(30));
    }

    #[test]
    fn rejects_invalid_watch_interval() {
        assert!(parse_config("watch_interval = \"5\"").is_err());
        assert!(parse_config("watch_interval = \"0s\"").is_err());
        assert!(parse_config("watch_interval = 5m").is_err());
    }
}
