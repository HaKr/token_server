use lazy_regex::regex;
use std::time::Duration;

use super::InvalidDuration;

pub fn format_duration(sec: u64) -> String {
    match sec {
        _ if sec < 60 || sec % 60 != 0 => format!("{}s", sec),
        _ if sec < 90 || sec % 3_600 != 0 => format!("{}min", sec / 60),
        _ => format!("{}h", sec / 3_600),
    }
}

fn parse_duration(str: &str) -> Result<Duration, InvalidDuration> {
    let splitter = regex!(r"^(\d+)(h|min|s|ms|μs|ns)$");

    splitter
        .captures(str)
        .map_or(Err(InvalidDuration::InvalidSyntax), |group| {
            let value = group[1].parse::<u64>()?;
            if value == 0 {
                Ok(Duration::ZERO)
            } else {
                match &group[2] {
                    "h" => Ok(Duration::from_secs(value * 3_600)),
                    "min" => Ok(Duration::from_secs(value * 60)),
                    "s" => Ok(Duration::from_secs(value)),
                    "ms" => Ok(Duration::from_millis(value)),
                    "μs" => Ok(Duration::from_micros(value)),
                    "ns" => Ok(Duration::from_nanos(value)),

                    sym => Err(InvalidDuration::UnsupportedDurationSymbol {
                        sym: sym.to_string(),
                    }),
                }
            }
        })
}

pub fn parse_duration_with_min_and_max(
    option: &str,
    min: u64,
    max: u64,
) -> Result<Duration, InvalidDuration> {
    let duration = parse_duration(option)?;

    let min_d = Duration::from_secs(min);
    let max_d = Duration::from_secs(max);
    if min_d <= duration && duration <= max_d {
        Ok(duration)
    } else {
        Err(InvalidDuration::DurationMustLieBetween {
            min: format_duration(min),
            max: format_duration(max),
        })
    }
}
