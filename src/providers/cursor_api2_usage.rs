use std::io::{self, Write};
use std::process::{Command, Stdio};

use crate::infra::diagnostics::Diagnostics;

const SECURITY_COMMAND: &str = "security";
const CURL_COMMAND: &str = "curl";
const CURSOR_USAGE_URL: &str =
    "https://api2.cursor.sh/aiserver.v1.DashboardService/GetCurrentPeriodUsage";

pub enum CursorApiUsageResult {
    Found(String),
    Unavailable(String),
}

pub fn get_usage_summary(diagnostics: &Diagnostics) -> io::Result<CursorApiUsageResult> {
    let token_output = Command::new(SECURITY_COMMAND)
        .args(["find-generic-password", "-s", "cursor-access-token", "-w"])
        .stdin(Stdio::null())
        .output();

    let token_output = match token_output {
        Ok(output) => output,
        Err(error) => {
            return Ok(CursorApiUsageResult::Unavailable(format!(
                "Cursor api2 usage unavailable: cannot read macOS Keychain token ({error})"
            )));
        }
    };

    if !token_output.status.success() {
        return Ok(CursorApiUsageResult::Unavailable(
            "Cursor api2 usage unavailable: token not found; run `cursor agent login`".to_string(),
        ));
    }

    let token = String::from_utf8_lossy(&token_output.stdout)
        .trim()
        .to_string();
    if token.is_empty() {
        return Ok(CursorApiUsageResult::Unavailable(
            "Cursor api2 usage unavailable: empty token; run `cursor agent login`".to_string(),
        ));
    }

    diagnostics.write_cursor_api_request(CURSOR_USAGE_URL)?;

    let curl = Command::new(CURL_COMMAND)
        .args(["-sS", "-X", "POST", CURSOR_USAGE_URL, "-K", "-", "-d", "{}"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut curl = match curl {
        Ok(child) => child,
        Err(error) => {
            drop(token);
            return Ok(CursorApiUsageResult::Unavailable(format!(
                "Cursor api2 usage unavailable: cannot run curl ({error})"
            )));
        }
    };

    if let Some(mut stdin) = curl.stdin.take() {
        stdin.write_all(
            format!(
                "header = \"Authorization: Bearer {token}\"\nheader = \"Content-Type: application/json\"\nheader = \"Connect-Protocol-Version: 1\"\n"
            )
            .as_bytes(),
        )?;
    }

    drop(token);

    let usage_output = match curl.wait_with_output() {
        Ok(output) => output,
        Err(error) => {
            return Ok(CursorApiUsageResult::Unavailable(format!(
                "Cursor api2 usage unavailable: cannot read curl output ({error})"
            )));
        }
    };

    let response = String::from_utf8_lossy(&usage_output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&usage_output.stderr).to_string();
    diagnostics.write_cursor_api_response(&response, &stderr)?;

    if !usage_output.status.success() {
        return Ok(CursorApiUsageResult::Unavailable(format!(
            "Cursor api2 usage unavailable: request failed with status {}",
            usage_output.status
        )));
    }

    if response.trim().is_empty() {
        return Ok(CursorApiUsageResult::Unavailable(
            "Cursor api2 usage unavailable: empty response".to_string(),
        ));
    }

    if response.contains("\"code\":\"unauthenticated\"")
        || response.contains("\"error\":\"unauthorized\"")
        || response.contains("Unauthorized")
    {
        return Ok(CursorApiUsageResult::Unavailable(
            "Cursor api2 usage unavailable: token rejected; run `cursor agent login`".to_string(),
        ));
    }

    match parse_cursor_api_usage(&response) {
        Some(summary) => Ok(CursorApiUsageResult::Found(summary)),
        None => Ok(CursorApiUsageResult::Unavailable(
            "Cursor api2 usage unavailable: response format is not recognized".to_string(),
        )),
    }
}

fn parse_cursor_api_usage(response: &str) -> Option<String> {
    let remaining = json_number_after_key(response, "remaining");
    let limit = json_number_after_key(response, "limit");
    let total_percent_used = json_number_after_key(response, "totalPercentUsed");
    let auto_percent_used = json_number_after_key(response, "autoPercentUsed");
    let api_percent_used = json_number_after_key(response, "apiPercentUsed");
    let billing_cycle_start = json_string_after_key(response, "billingCycleStart")
        .and_then(|value| value.parse::<i64>().ok())
        .or_else(|| json_number_after_key(response, "billingCycleStart").map(|value| value as i64));
    let billing_cycle_end = json_string_after_key(response, "billingCycleEnd")
        .and_then(|value| value.parse::<i64>().ok())
        .or_else(|| json_number_after_key(response, "billingCycleEnd").map(|value| value as i64));
    let display_message = json_string_after_key(response, "displayMessage");

    if remaining.is_none()
        && limit.is_none()
        && total_percent_used.is_none()
        && billing_cycle_start.is_none()
        && billing_cycle_end.is_none()
    {
        return None;
    }

    let mut summary = String::from("Cursor usage:\n");

    if let (Some(remaining), Some(limit)) = (remaining, limit) {
        let used = (limit - remaining).max(0.0);
        summary.push_str(&format!(
            "Plan usage: {} / {}",
            format_cents(used),
            format_cents(limit)
        ));
        if let Some(percent) = total_percent_used {
            summary.push_str(&format!(" ({}%)", format_number(percent)));
        }
        summary.push('\n');
        summary.push_str(&format!("Remaining: {}\n", format_cents(remaining)));
    } else if let Some(percent) = total_percent_used {
        summary.push_str(&format!("Plan usage: {}% used\n", format_number(percent)));
    }

    if auto_percent_used.is_some() || api_percent_used.is_some() {
        summary.push_str(&format!(
            "Auto: {}% | API models: {}%\n",
            auto_percent_used
                .map(format_number)
                .unwrap_or_else(|| "n/a".to_string()),
            api_percent_used
                .map(format_number)
                .unwrap_or_else(|| "n/a".to_string())
        ));
    }

    if let (Some(start), Some(end)) = (billing_cycle_start, billing_cycle_end) {
        summary.push_str(&format!(
            "Cycle: {} -> {}\n",
            format_unix_ms_date(start),
            format_unix_ms_date(end)
        ));
    }

    if let Some(message) = display_message.filter(|value| !value.is_empty()) {
        summary.push_str(&message);
        summary.push('\n');
    }

    Some(summary)
}

fn json_number_after_key(input: &str, key: &str) -> Option<f64> {
    let mut rest = input;
    let needle = format!("\"{key}\"");

    loop {
        let key_index = rest.find(&needle)?;
        let after_key = &rest[key_index + needle.len()..];
        let colon_index = after_key.find(':')?;
        let after_colon = after_key[colon_index + 1..].trim_start();
        let number_len = after_colon
            .chars()
            .take_while(|character| {
                character.is_ascii_digit()
                    || *character == '-'
                    || *character == '+'
                    || *character == '.'
                    || *character == 'e'
                    || *character == 'E'
            })
            .map(char::len_utf8)
            .sum::<usize>();

        if number_len > 0 {
            return after_colon[..number_len].parse::<f64>().ok();
        }

        rest = &after_colon[after_colon.chars().next()?.len_utf8()..];
    }
}

fn json_string_after_key(input: &str, key: &str) -> Option<String> {
    let mut rest = input;
    let needle = format!("\"{key}\"");

    loop {
        let key_index = rest.find(&needle)?;
        let after_key = &rest[key_index + needle.len()..];
        let colon_index = after_key.find(':')?;
        let after_colon = after_key[colon_index + 1..].trim_start();
        if let Some(value) = parse_json_string(after_colon) {
            return Some(value);
        }

        rest = &after_colon[after_colon.chars().next()?.len_utf8()..];
    }
}

fn parse_json_string(input: &str) -> Option<String> {
    let mut chars = input.chars();
    if chars.next()? != '"' {
        return None;
    }

    let mut value = String::new();
    let mut escaped = false;
    for character in chars {
        if escaped {
            value.push(match character {
                '"' => '"',
                '\\' => '\\',
                '/' => '/',
                'b' => '\u{0008}',
                'f' => '\u{000c}',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => other,
            });
            escaped = false;
            continue;
        }

        match character {
            '\\' => escaped = true,
            '"' => return Some(value),
            other => value.push(other),
        }
    }

    None
}

fn format_cents(value: f64) -> String {
    format!("${:.2}", value / 100.0)
}

fn format_number(value: f64) -> String {
    if (value.fract()).abs() < f64::EPSILON {
        format!("{}", value as i64)
    } else {
        format!("{value:.2}")
    }
}

fn format_unix_ms_date(value: i64) -> String {
    let seconds = value.div_euclid(1000);
    let days = seconds.div_euclid(86_400);
    civil_date_from_days(days)
        .map(|(year, month, day)| format!("{year:04}-{month:02}-{day:02}"))
        .unwrap_or_else(|| value.to_string())
}

fn civil_date_from_days(days_since_unix_epoch: i64) -> Option<(i32, u32, u32)> {
    let days = days_since_unix_epoch.checked_add(719_468)?;
    let era = if days >= 0 { days } else { days - 146_096 }.div_euclid(146_097);
    let day_of_era = days - era * 146_097;
    let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524
        - day_of_era / 146_096)
        .div_euclid(365);
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2).div_euclid(153);
    let day = day_of_year - (153 * month_prime + 2).div_euclid(5) + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let adjusted_year = year + if month <= 2 { 1 } else { 0 };

    Some((adjusted_year as i32, month as u32, day as u32))
}
