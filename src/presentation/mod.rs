mod common;
mod limits;
mod usage;

pub use common::{ColorConfig, ProviderBlock};
pub use limits::limits_block;
pub use usage::usage_block;

pub fn format_raw_output(data: &crate::types::SourceData) -> String {
    match data.raw.as_deref() {
        Some(raw) if !raw.trim().is_empty() => raw.to_string(),
        _ if !data.structured.raw_data_available => data
            .structured
            .status
            .message
            .clone()
            .unwrap_or_else(|| "raw data unavailable".to_string()),
        _ => data
            .structured
            .status
            .message
            .clone()
            .unwrap_or_else(|| "raw data unavailable".to_string()),
    }
}

pub fn format_structured_output(data: &crate::types::SourceData) -> String {
    serde_json::to_string_pretty(&data.structured).unwrap_or_else(|error| {
        format!("failed to serialize structured data: {error}")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        AccountInfo, ActivityUsage, LimitInfo, ModelUsage, MoneyUsage, SourceStatus,
        StructuredSourceInfo, TokenUsage, UsageInfo,
    };

    fn sample_limits_info() -> StructuredSourceInfo {
        StructuredSourceInfo {
            provider: "codex".to_string(),
            source: "codex_cli".to_string(),
            source_link: "docs/get-info/providers/codex.md".to_string(),
            status: SourceStatus {
                data_available: true,
                access_available: true,
                message: None,
            },
            raw_data_available: true,
            collected_at: Some("2026-06-30T21:41:00Z".to_string()),
            data_as_of: Some("Jul 3, 21:41 UTC-2".to_string()),
            account: AccountInfo {
                credits_remaining: Some(344.2),
                ..Default::default()
            },
            limits: vec![
                LimitInfo {
                    name: "5h limit".to_string(),
                    window_label: Some("5h".to_string()),
                    window_minutes: Some(300),
                    resets_at: Some("Jun 30, 21:41 UTC-2".to_string()),
                    used_percent: Some(92.0),
                    remaining_percent: Some(8.0),
                    ..Default::default()
                },
                LimitInfo {
                    name: "Weekly limit".to_string(),
                    window_label: Some("7d".to_string()),
                    window_minutes: Some(10080),
                    resets_at: Some("Jul 3, 21:41 UTC-2".to_string()),
                    used_percent: Some(46.0),
                    remaining_percent: Some(54.0),
                    ..Default::default()
                },
            ],
            usage: UsageInfo::default(),
            diagnostics: Vec::new(),
        }
    }

    fn sample_usage_info() -> StructuredSourceInfo {
        StructuredSourceInfo {
            provider: "codex".to_string(),
            source: "codex_local".to_string(),
            source_link: "docs/get-info/providers/codex.md".to_string(),
            status: SourceStatus {
                data_available: true,
                access_available: true,
                message: None,
            },
            raw_data_available: true,
            collected_at: Some("2026-06-30T21:41:00Z".to_string()),
            data_as_of: Some("Jul 3, 21:41 UTC-2".to_string()),
            account: AccountInfo::default(),
            limits: Vec::new(),
            usage: UsageInfo {
                tokens: TokenUsage {
                    input: Some(120_000),
                    cached_input: Some(80_000),
                    output: Some(30_000),
                    total: Some(230_000),
                    ..Default::default()
                },
                activity: ActivityUsage {
                    sessions_count: Some(14),
                    turns_count: Some(128),
                    latest_activity_at: Some("Jul 3, 21:41 UTC-2".to_string()),
                    ..Default::default()
                },
                models: ModelUsage {
                    top_model: Some("gpt-5".to_string()),
                },
                money: MoneyUsage {
                    used_amount: Some(12.4),
                    currency: Some("usd".to_string()),
                    ..Default::default()
                },
            },
            diagnostics: Vec::new(),
        }
    }

    #[test]
    fn limits_block_renders_rows_credits_and_data_as_of() {
        let block = limits_block(&sample_limits_info(), &ColorConfig { enabled: false });

        assert_eq!(block.provider_label, "CODEX");
        assert!(block.body.contains("5h   "));
        assert!(block.body.contains("8% left | reset Jun 30, 21:41 UTC-2"));
        assert!(block.body.contains("54% left | reset Jul 3, 21:41 UTC-2"));
        assert!(block.body.contains("344.2 credits available"));
        assert!(block.body.contains("Data as of: Jul 3, 21:41 UTC-2"));
    }

    #[test]
    fn limits_block_renders_unavailable_state() {
        let info = StructuredSourceInfo {
            status: SourceStatus {
                data_available: false,
                access_available: false,
                message: Some("not logged in".to_string()),
            },
            data_as_of: None,
            ..sample_limits_info()
        };
        let block = limits_block(&info, &ColorConfig { enabled: false });

        assert!(block.body.contains("Unavailable: not logged in"));
        assert!(block.body.contains("Data as of: unknown"));
    }

    #[test]
    fn limits_block_renders_no_limit_data_message() {
        let info = StructuredSourceInfo {
            limits: Vec::new(),
            account: AccountInfo::default(),
            ..sample_limits_info()
        };
        let block = limits_block(&info, &ColorConfig { enabled: false });

        assert!(block.body.contains("No limit data from this source"));
        assert!(block.body.contains("Data as of: Jul 3, 21:41 UTC-2"));
    }

    #[test]
    fn usage_block_renders_available_rows() {
        let block = usage_block(&sample_usage_info());

        assert_eq!(block.provider_label, "CODEX");
        assert!(block.body.contains("Tokens        input 120,000 | cached 80,000 | output 30,000 | total 230,000"));
        assert!(block.body.contains("Activity      14 sessions | 128 turns | latest Jul 3, 21:41 UTC-2"));
        assert!(block.body.contains("Models        top: gpt-5"));
        assert!(block.body.contains("Money         $12.40 used"));
        assert!(block.body.contains("Data as of: Jul 3, 21:41 UTC-2"));
    }

    #[test]
    fn limit_bar_uses_twenty_five_characters_without_color() {
        let bar = common::visible_limit_bar(54.0);
        assert_eq!(bar.chars().count(), 25);
        assert_eq!(bar.chars().filter(|character| *character == '■').count(), 14);
        assert_eq!(bar.chars().filter(|character| *character == '□').count(), 11);
        assert!(!bar.contains('◧'));
    }

    #[test]
    fn limits_block_shows_one_decimal_place_for_fractional_percent() {
        let mut info = sample_limits_info();
        info.limits = vec![LimitInfo {
            name: "plan_usage".to_string(),
            window_label: Some("plan".to_string()),
            used_percent: Some(37.5),
            remaining_percent: Some(62.5),
            resets_at: Some("Jul 3, 21:41 UTC-2".to_string()),
            ..Default::default()
        }];

        let block = limits_block(&info, &ColorConfig { enabled: false });

        assert!(block.body.contains("62.5% left"));
    }

    #[test]
    fn format_percent_omits_trailing_zero() {
        assert_eq!(common::format_percent(84.0), "84");
        assert_eq!(common::format_percent(62.5), "62.5");
    }
}
