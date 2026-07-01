use std::{env, fs, io, path::PathBuf};

use crate::types::Source;

pub struct Config {
    pub default_sources: Vec<Source>,
}

const DEFAULT_CONFIG: &str = "\
default_sources = [
  \"codex_local\",
  \"claude_local\",
  \"cursor_api2\"
]
";

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
            unknown => return Err(format!("unknown config key `{unknown}`")),
        }
    }

    Ok(Config {
        default_sources: default_sources.unwrap_or_default(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_config_sources() {
        let parsed = parse_config(DEFAULT_CONFIG).expect("default config should parse");

        assert_eq!(
            parsed.default_sources,
            vec![Source::CodexLocal, Source::ClaudeLocal, Source::CursorApi2]
        );
    }

    #[test]
    fn parse_config_accepts_claude_hook() {
        let parsed = parse_config("default_sources = [\"claude_hook\"]")
            .expect("claude_hook should be accepted");

        assert_eq!(parsed.default_sources, vec![Source::ClaudeHook]);
    }
}
