use crate::types::{LimitInfo, StructuredSourceInfo};

use super::common::{
    format_data_as_of, format_decimal, format_percent, format_unavailable_block, normalize_percent,
    provider_label, render_limit_bar, window_label_for_display, ColorConfig, ProviderBlock,
};

pub fn limits_block(info: &StructuredSourceInfo, color: &ColorConfig) -> ProviderBlock {
    ProviderBlock {
        provider_label: provider_label(info),
        body: format_limits_body(info, color),
    }
}

fn format_limits_body(info: &StructuredSourceInfo, color: &ColorConfig) -> String {
    if !info.status.access_available {
        return format_unavailable_block(info);
    }

    let limit_rows = info
        .limits
        .iter()
        .filter_map(|limit| format_limit_row(limit, color))
        .collect::<Vec<_>>();

    if limit_rows.is_empty() {
        let mut body = String::from("No limit data from this source\n");
        body.push_str(&format_data_as_of(info));
        return body;
    }

    let mut body = limit_rows.join("\n");
    body.push('\n');

    if let Some(credits) = info.account.credits_remaining {
        body.push_str(&format!("{} credits available\n", format_decimal(credits)));
    }

    body.push_str(&format_data_as_of(info));
    body
}

fn format_limit_row(limit: &LimitInfo, color: &ColorConfig) -> Option<String> {
    let remaining_percent = limit.remaining_percent.or_else(|| {
        limit
            .used_percent
            .map(|used| (100.0 - used).clamp(0.0, 100.0))
    })?;
    let remaining_display = normalize_percent(remaining_percent);

    let window = format!("{:<4}", window_label_for_display(limit));
    let bar = render_limit_bar(remaining_display, color);
    let left = format!("{}% left", format_percent(remaining_percent));
    let reset = limit
        .resets_at
        .as_deref()
        .map(|value| format!(" | reset {value}"))
        .unwrap_or_default();

    Some(format!("{window} {bar} {left:>8}{reset}"))
}
