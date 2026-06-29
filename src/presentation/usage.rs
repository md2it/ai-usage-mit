use crate::types::StructuredSourceInfo;

use super::common::{
    format_data_as_of, format_decimal, format_number, format_unavailable_block, provider_label,
    ProviderBlock,
};

pub fn usage_block(info: &StructuredSourceInfo) -> ProviderBlock {
    ProviderBlock {
        provider_label: provider_label(info),
        body: format_usage_body(info),
    }
}

fn format_usage_body(info: &StructuredSourceInfo) -> String {
    if !info.status.access_available {
        return format_unavailable_block(info);
    }

    let mut rows = Vec::new();

    if let Some(line) = format_tokens_row(info) {
        rows.push(line);
    }
    if let Some(line) = format_activity_row(info) {
        rows.push(line);
    }
    if let Some(line) = format_models_row(info) {
        rows.push(line);
    }
    if let Some(line) = format_money_row(info) {
        rows.push(line);
    }

    if rows.is_empty() && !info.status.data_available {
        return format_unavailable_block(info);
    }

    let mut body = rows.join("\n");
    if !body.is_empty() {
        body.push('\n');
    }
    body.push('\n');
    body.push_str(&format_data_as_of(info));
    body
}

fn format_tokens_row(info: &StructuredSourceInfo) -> Option<String> {
    let tokens = &info.usage.tokens;
    let mut parts = Vec::new();

    if let Some(value) = tokens.input {
        parts.push(format!("input {}", format_number(value)));
    }
    if let Some(value) = tokens.cached_input {
        parts.push(format!("cached {}", format_number(value)));
    }
    if let Some(value) = tokens.output {
        parts.push(format!("output {}", format_number(value)));
    }
    if let Some(value) = tokens.reasoning_output {
        parts.push(format!("reasoning {}", format_number(value)));
    }
    if let Some(value) = tokens.cache_read {
        parts.push(format!("cache read {}", format_number(value)));
    }
    if let Some(value) = tokens.cache_write {
        parts.push(format!("cache write {}", format_number(value)));
    }
    if let Some(value) = tokens.total {
        parts.push(format!("total {}", format_number(value)));
    }

    if parts.is_empty() {
        None
    } else {
        Some(format!("Tokens        {}", parts.join(" | ")))
    }
}

fn format_activity_row(info: &StructuredSourceInfo) -> Option<String> {
    let activity = &info.usage.activity;
    let mut parts = Vec::new();

    if let Some(value) = activity.sessions_count {
        parts.push(format!(
            "{} session{}",
            format_number(value),
            if value == 1 { "" } else { "s" }
        ));
    }
    if let Some(value) = activity.turns_count {
        parts.push(format!(
            "{} turn{}",
            format_number(value),
            if value == 1 { "" } else { "s" }
        ));
    }
    if let Some(value) = activity.files_count {
        parts.push(format!(
            "{} file{}",
            format_number(value),
            if value == 1 { "" } else { "s" }
        ));
    }
    if let Some(value) = activity.events_count {
        parts.push(format!(
            "{} event{}",
            format_number(value),
            if value == 1 { "" } else { "s" }
        ));
    }
    if let Some(value) = activity.latest_activity_at.as_deref() {
        parts.push(format!("latest {value}"));
    }

    if parts.is_empty() {
        None
    } else {
        Some(format!("Activity      {}", parts.join(" | ")))
    }
}

fn format_models_row(info: &StructuredSourceInfo) -> Option<String> {
    info.usage
        .models
        .top_model
        .as_deref()
        .map(|model| format!("Models        top: {model}"))
}

fn format_money_row(info: &StructuredSourceInfo) -> Option<String> {
    let money = &info.usage.money;
    let mut parts = Vec::new();

    if let Some(value) = money.used_amount {
        parts.push(format_used_money(value, money.currency.as_deref()));
    }
    if let Some(value) = money.remaining_amount {
        parts.push(format_remaining_money(value, money.currency.as_deref()));
    }
    if let Some(value) = money.total_amount {
        parts.push(format_total_money(value, money.currency.as_deref()));
    }

    if parts.is_empty() {
        None
    } else {
        Some(format!("Money         {}", parts.join(" | ")))
    }
}

fn format_used_money(amount: f64, currency: Option<&str>) -> String {
    match currency {
        Some("USD") | Some("usd") | None => format!("${} used", format_money_amount(amount)),
        Some(code) => format!("{amount} {code} used"),
    }
}

fn format_remaining_money(amount: f64, currency: Option<&str>) -> String {
    match currency {
        Some("USD") | Some("usd") | None => format!("${} remaining", format_money_amount(amount)),
        Some(code) => format!("{amount} {code} remaining"),
    }
}

fn format_total_money(amount: f64, currency: Option<&str>) -> String {
    match currency {
        Some("USD") | Some("usd") | None => format!("${} total", format_money_amount(amount)),
        Some(code) => format!("{amount} {code} total"),
    }
}

fn format_money_amount(amount: f64) -> String {
    if amount.fract().abs() < f64::EPSILON {
        format_decimal(amount)
    } else {
        format!("{amount:.2}")
    }
}
