use chrono::{
    DateTime, Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime, Offset, TimeZone, Utc,
};
use chrono_tz::Tz;

use crate::types::StructuredSourceInfo;

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

pub struct TimeContext {
    reference: DateTime<Local>,
}

impl TimeContext {
    pub fn from_structured(info: &StructuredSourceInfo) -> Self {
        Self {
            reference: info
                .collected_at
                .as_deref()
                .and_then(parse_instant_reference)
                .or_else(|| info.data_as_of.as_deref().and_then(parse_instant_reference))
                .unwrap_or_else(Local::now),
        }
    }
}

pub fn format_user_timestamp(value: &str, context: &TimeContext) -> String {
    parse_to_local(value, context)
        .map(format_local_datetime)
        .unwrap_or_else(|| value.to_string())
}

pub fn format_local_datetime(dt: DateTime<Local>) -> String {
    let month = MONTHS.get(dt.month0() as usize).copied().unwrap_or("???");
    format!(
        "{} {:2}, {} {}",
        month,
        dt.day(),
        dt.format("%H:%M"),
        format_utc_offset(dt.offset().fix().local_minus_utc())
    )
}

fn format_utc_offset(offset_seconds: i32) -> String {
    let sign = if offset_seconds >= 0 { '+' } else { '-' };
    let absolute = offset_seconds.abs();
    let hours = absolute / 3600;
    let minutes = (absolute % 3600) / 60;
    if minutes == 0 {
        format!("UTC{sign}{hours}")
    } else {
        format!("UTC{sign}{hours}:{minutes:02}")
    }
}

fn parse_instant_reference(value: &str) -> Option<DateTime<Local>> {
    parse_iso_or_unix(value)
}

fn parse_to_local(value: &str, context: &TimeContext) -> Option<DateTime<Local>> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(parsed) = parse_iso_or_unix(trimmed) {
        return Some(parsed);
    }

    let (body, timezone) = split_timezone_suffix(trimmed);
    if let Some(parsed) = parse_source_specific(body, timezone, context) {
        return Some(parsed);
    }

    None
}

fn parse_iso_or_unix(value: &str) -> Option<DateTime<Local>> {
    if let Ok(parsed) = DateTime::parse_from_rfc3339(value) {
        return Some(parsed.with_timezone(&Local));
    }
    for format in [
        "%Y-%m-%dT%H:%M:%SZ",
        "%Y-%m-%dT%H:%M:%S%.fZ",
        "%Y-%m-%dT%H:%M:%S%.3fZ",
    ] {
        if let Ok(parsed) = NaiveDateTime::parse_from_str(value, format) {
            return Some(Utc.from_utc_datetime(&parsed).with_timezone(&Local));
        }
    }
    if value.chars().all(|character| character.is_ascii_digit()) {
        let seconds = value.parse::<i64>().ok()?;
        return Utc
            .timestamp_opt(seconds, 0)
            .single()
            .map(|parsed| parsed.with_timezone(&Local));
    }

    None
}

fn split_timezone_suffix(value: &str) -> (&str, Option<Tz>) {
    let Some(open) = value.rfind('(') else {
        return (value, None);
    };
    if !value.ends_with(')') {
        return (value, None);
    }

    let name = value[open + 1..value.len() - 1].trim();
    let body = value[..open].trim();
    (body, name.parse::<Tz>().ok())
}

fn parse_source_specific(
    body: &str,
    timezone: Option<Tz>,
    context: &TimeContext,
) -> Option<DateTime<Local>> {
    if let Some(parsed) = parse_on_date_format(body, timezone, context) {
        return Some(parsed);
    }
    if let Some(parsed) = parse_month_day_at_time(body, timezone, context) {
        return Some(parsed);
    }
    if let Some(parsed) = parse_time_only(body, timezone, context) {
        return Some(parsed);
    }

    None
}

fn parse_on_date_format(
    body: &str,
    timezone: Option<Tz>,
    context: &TimeContext,
) -> Option<DateTime<Local>> {
    let (time_part, date_part) = body.split_once(" on ")?;
    let date = parse_day_month(date_part.trim(), context.reference.year())?;
    let time = parse_clock_time(time_part.trim())?;
    assemble_local_datetime(date, time, timezone, context, DateRollPolicy::YearIfPast)
}

fn parse_month_day_at_time(
    body: &str,
    timezone: Option<Tz>,
    context: &TimeContext,
) -> Option<DateTime<Local>> {
    let (date_part, time_part) = body.split_once(" at ")?;
    let date = parse_month_day(date_part.trim(), context.reference.year())?;
    let time = parse_clock_time(time_part.trim())?;
    assemble_local_datetime(date, time, timezone, context, DateRollPolicy::YearIfPast)
}

fn parse_time_only(
    body: &str,
    timezone: Option<Tz>,
    context: &TimeContext,
) -> Option<DateTime<Local>> {
    let time = parse_clock_time(body)?;
    assemble_local_datetime(
        context.reference.date_naive(),
        time,
        timezone,
        context,
        DateRollPolicy::DayIfPast,
    )
}

enum DateRollPolicy {
    DayIfPast,
    YearIfPast,
}

fn assemble_local_datetime(
    mut date: NaiveDate,
    time: NaiveTime,
    timezone: Option<Tz>,
    context: &TimeContext,
    roll: DateRollPolicy,
) -> Option<DateTime<Local>> {
    let mut local = localize_naive(date, time, timezone)?;

    match roll {
        DateRollPolicy::DayIfPast if local <= context.reference => {
            date = date.succ_opt()?;
            local = localize_naive(date, time, timezone)?;
        }
        DateRollPolicy::YearIfPast if local <= context.reference => {
            date = NaiveDate::from_ymd_opt(date.year() + 1, date.month(), date.day())?;
            local = localize_naive(date, time, timezone)?;
        }
        _ => {}
    }

    Some(local)
}

fn localize_naive(
    date: NaiveDate,
    time: NaiveTime,
    timezone: Option<Tz>,
) -> Option<DateTime<Local>> {
    let naive = NaiveDateTime::new(date, time);
    if let Some(timezone) = timezone {
        return timezone
            .from_local_datetime(&naive)
            .single()
            .map(|parsed| parsed.with_timezone(&Local));
    }

    Local
        .from_local_datetime(&naive)
        .single()
        .map(|parsed| parsed.with_timezone(&Local))
}

fn parse_day_month(value: &str, year: i32) -> Option<NaiveDate> {
    let mut parts = value.split_whitespace();
    let day = parts.next()?.parse::<u32>().ok()?;
    let month = parse_month_name(parts.next()?)?;
    NaiveDate::from_ymd_opt(year, month, day)
}

fn parse_month_day(value: &str, year: i32) -> Option<NaiveDate> {
    let mut parts = value.split_whitespace();
    let month = parse_month_name(parts.next()?)?;
    let day = parts.next()?.parse::<u32>().ok()?;
    NaiveDate::from_ymd_opt(year, month, day)
}

fn parse_month_name(value: &str) -> Option<u32> {
    let normalized = value.trim_end_matches('.').to_ascii_lowercase();
    match normalized.as_str() {
        "jan" | "january" => Some(1),
        "feb" | "february" => Some(2),
        "mar" | "march" => Some(3),
        "apr" | "april" => Some(4),
        "may" => Some(5),
        "jun" | "june" => Some(6),
        "jul" | "july" => Some(7),
        "aug" | "august" => Some(8),
        "sep" | "sept" | "september" => Some(9),
        "oct" | "october" => Some(10),
        "nov" | "november" => Some(11),
        "dec" | "december" => Some(12),
        _ => None,
    }
}

fn parse_clock_time(value: &str) -> Option<NaiveTime> {
    let trimmed = value.trim();
    if let Ok(parsed) = NaiveTime::parse_from_str(trimmed, "%H:%M") {
        return Some(parsed);
    }

    let lower = trimmed.to_ascii_lowercase();
    let (body, pm, is_12h) = if let Some(body) = lower.strip_suffix("am") {
        (body, false, true)
    } else if let Some(body) = lower.strip_suffix("pm") {
        (body, true, true)
    } else {
        (lower.as_str(), false, false)
    };

    let body = body.trim();
    let (hour, minute) = match body.split_once(':') {
        Some((hour, minute)) => (hour.parse::<u32>().ok()?, minute.parse::<u32>().ok()?),
        None => (body.parse::<u32>().ok()?, 0),
    };

    let hour = if is_12h {
        if pm {
            if hour == 12 {
                12
            } else {
                hour + 12
            }
        } else if hour == 12 {
            0
        } else {
            hour
        }
    } else {
        hour
    };

    NaiveTime::from_hms_opt(hour, minute, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed_context(iso: &str) -> TimeContext {
        TimeContext {
            reference: parse_iso_or_unix(iso).expect("reference should parse"),
        }
    }

    #[test]
    fn parses_iso_utc_timestamp() {
        assert!(parse_iso_or_unix("2026-06-29T23:09:29Z").is_some());
    }

    #[test]
    fn formats_iso_utc_in_local_style_with_offset_label() {
        let context = fixed_context("2026-06-29T20:00:00Z");
        let formatted = format_user_timestamp("2026-06-29T23:09:29Z", &context);

        assert!(formatted.contains("UTC"));
        assert!(!formatted.contains("T23:"));
        assert!(!formatted.ends_with('Z'));
    }

    #[test]
    fn formats_claude_cli_session_reset_with_source_timezone() {
        let context = fixed_context("2026-06-29T20:00:00Z");
        let formatted = format_user_timestamp("2:20am (Asia/Nicosia)", &context);

        assert!(formatted.starts_with("Jun 30, "));
        assert!(formatted.contains("02:20"));
        assert!(formatted.contains("UTC"));
    }

    #[test]
    fn formats_claude_cli_week_reset_with_date() {
        let context = fixed_context("2026-06-29T20:00:00Z");
        let formatted = format_user_timestamp("Jun 30 at 1pm (Asia/Nicosia)", &context);

        assert!(formatted.starts_with("Jun 30, "));
        assert!(formatted.contains("13:00"));
        assert!(formatted.contains("UTC"));
    }

    #[test]
    fn formats_codex_cli_weekly_reset_pattern() {
        let context = fixed_context("2026-06-29T20:00:00Z");
        let formatted = format_user_timestamp("02:59 on 6 Jul", &context);

        assert!(formatted.starts_with("Jul  6, "));
        assert!(formatted.contains("02:59"));
        assert!(formatted.contains("UTC"));
    }

    #[test]
    fn format_utc_offset_renders_half_hour_offsets() {
        assert_eq!(format_utc_offset(7200), "UTC+2");
        assert_eq!(format_utc_offset(19_800), "UTC+5:30");
        assert_eq!(format_utc_offset(-18_000), "UTC-5");
    }

    #[test]
    fn formatted_timestamps_align_clock_time_after_comma() {
        let context = fixed_context("2026-06-29T20:00:00Z");
        let single_digit_day = format_user_timestamp("02:59 on 6 Jul", &context);
        let double_digit_day = format_user_timestamp("Jun 30 at 1pm (Asia/Nicosia)", &context);

        fn time_start(value: &str) -> usize {
            value.find(", ").expect("comma") + 2
        }

        assert_eq!(time_start(&single_digit_day), time_start(&double_digit_day));
        assert!(single_digit_day.contains("Jul  6, "));
    }
}
