use crate::types::{LimitInfo, StructuredSourceInfo};

pub struct ProviderBlock {
    pub provider_label: String,
    pub body: String,
}

pub struct ColorConfig {
    pub enabled: bool,
}

impl ColorConfig {
    pub fn from_env(is_tty: bool) -> Self {
        let disabled = std::env::var_os("NO_COLOR").is_some();
        Self {
            enabled: is_tty && !disabled,
        }
    }
}

pub fn provider_label(info: &StructuredSourceInfo) -> String {
    info.provider.to_ascii_uppercase()
}

pub fn format_data_as_of(info: &StructuredSourceInfo) -> String {
    match info.data_as_of.as_deref() {
        Some(value) => format!("Data as of: {value}"),
        None => "Data as of: unknown".to_string(),
    }
}

pub fn format_unavailable_block(info: &StructuredSourceInfo) -> String {
    let message = info
        .status
        .message
        .as_deref()
        .unwrap_or("unavailable");
    format!("Unavailable: {message}\n{}", format_data_as_of(info))
}

pub fn format_decimal(value: f64) -> String {
    let rounded = (value * 10.0).round() / 10.0;
    if rounded.fract() == 0.0 {
        format!("{rounded:.0}")
    } else {
        format!("{rounded:.1}")
    }
}

pub fn format_number(value: u64) -> String {
    let digits = value.to_string();
    let mut formatted = String::new();

    for (index, character) in digits.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            formatted.push(',');
        }
        formatted.push(character);
    }

    formatted.chars().rev().collect()
}

pub fn window_label_for_display(limit: &LimitInfo) -> String {
    if let Some(minutes) = limit.window_minutes {
        return compact_window_from_minutes(minutes);
    }

    if let Some(label) = limit.window_label.as_deref() {
        let compact = compact_window_label(label);
        if compact.chars().count() <= 4 {
            return compact;
        }
    }

    compact_name_label(&limit.name)
}

fn compact_window_from_minutes(minutes: u64) -> String {
    match minutes {
        300 => "5h".to_string(),
        10080 => "7d".to_string(),
        _ => format!("{minutes}m"),
    }
}

fn compact_window_label(label: &str) -> String {
    let normalized = label.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "5h" | "5-hour window" | "5 hour" | "five_hour" | "primary window" => "5h".to_string(),
        "7d" | "7-day window" | "7 day" | "seven_day" | "weekly" | "secondary window" => {
            "7d".to_string()
        }
        _ if normalized.contains("5") && normalized.contains("hour") => "5h".to_string(),
        _ if normalized.contains("7") && normalized.contains("day") => "7d".to_string(),
        _ if normalized.contains("week") => "7d".to_string(),
        _ => label.to_string(),
    }
}

fn compact_name_label(name: &str) -> String {
    match name {
        "5h limit" => "5h".to_string(),
        "Weekly limit" => "7d".to_string(),
        "auto" => "auto".to_string(),
        "api_models" => "api".to_string(),
        "plan_usage" => "plan".to_string(),
        other => {
            let trimmed = other.trim();
            if trimmed.chars().count() <= 4 {
                trimmed.to_string()
            } else {
                trimmed.chars().take(4).collect()
            }
        }
    }
}

pub fn color_for_remaining(remaining_percent: f64, color: &ColorConfig) -> &'static str {
    if !color.enabled {
        return "";
    }

    if remaining_percent >= 75.0 {
        "\x1b[32m"
    } else if remaining_percent >= 50.0 {
        "\x1b[33m"
    } else if remaining_percent >= 25.0 {
        "\x1b[38;5;208m"
    } else if remaining_percent >= 10.0 {
        "\x1b[31m"
    } else {
        "\x1b[91m"
    }
}

pub const COLOR_RESET: &str = "\x1b[0m";

pub fn render_limit_bar(remaining_percent: f64, color: &ColorConfig) -> String {
    let visible = visible_limit_bar(remaining_percent);
    colorize_limit_bar(&visible, remaining_percent, color)
}

pub fn visible_limit_bar(remaining_percent: f64) -> String {
    let clamped = remaining_percent.clamp(0.0, 100.0);
    let total_units = 50;
    let filled_units = ((clamped / 100.0) * total_units as f64).round() as usize;
    let full_blocks = filled_units / 2;
    let half_block = filled_units % 2 == 1;
    let mut bar = String::with_capacity(25);

    for _ in 0..full_blocks {
        bar.push('■');
    }
    if half_block {
        bar.push('◧');
    }
    let empty_blocks = 25_usize.saturating_sub(full_blocks + usize::from(half_block));
    for _ in 0..empty_blocks {
        bar.push('□');
    }

    bar
}

fn colorize_limit_bar(visible: &str, remaining_percent: f64, color: &ColorConfig) -> String {
    if !color.enabled {
        return visible.to_string();
    }

    let color_code = color_for_remaining(remaining_percent, color);
    if color_code.is_empty() {
        return visible.to_string();
    }

    let mut colored = String::new();
    let mut in_fill = false;
    for character in visible.chars() {
        let is_filled = matches!(character, '■' | '◧');
        if is_filled && !in_fill {
            colored.push_str(color_code);
            in_fill = true;
        } else if !is_filled && in_fill {
            colored.push_str(COLOR_RESET);
            in_fill = false;
        }
        colored.push(character);
    }
    if in_fill {
        colored.push_str(COLOR_RESET);
    }

    colored
}
