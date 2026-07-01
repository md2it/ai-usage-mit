use crate::types::{LimitInfo, StructuredSourceInfo};

use super::common::{
    format_data_as_of, format_decimal, format_percent, format_unavailable_block, normalize_percent,
    pad_visible_left, pad_visible_right, provider_label, render_limit_bar,
    window_label_for_display, ColorConfig, ProviderBlock, LIMIT_BAR_WIDTH, LIMIT_LEFT_WIDTH,
    LIMIT_WINDOW_WIDTH,
};
use super::time::{format_user_timestamp, TimeContext};

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

    let time_context = TimeContext::from_structured(info);
    let limit_rows = info
        .limits
        .iter()
        .filter_map(|limit| format_limit_row(limit, color, &time_context))
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

fn format_limit_row(
    limit: &LimitInfo,
    color: &ColorConfig,
    time_context: &TimeContext,
) -> Option<String> {
    let remaining_percent = limit.remaining_percent.or_else(|| {
        limit
            .used_percent
            .map(|used| (100.0 - used).clamp(0.0, 100.0))
    })?;
    let remaining_display = normalize_percent(remaining_percent);

    let window = pad_visible_right(
        &format!(
            "{:<width$}",
            window_label_for_display(limit),
            width = LIMIT_WINDOW_WIDTH
        ),
        LIMIT_WINDOW_WIDTH,
    );
    let bar = pad_visible_right(&render_limit_bar(remaining_display, color), LIMIT_BAR_WIDTH);
    let left = pad_visible_left(
        &format!("{}% left", format_percent(remaining_percent)),
        LIMIT_LEFT_WIDTH,
    );

    let mut line = format!("{window} {bar} {left}");
    if let Some(value) = limit.resets_at.as_deref() {
        line.push_str(" | reset ");
        line.push_str(&format_user_timestamp(value, time_context));
    }

    Some(line)
}
