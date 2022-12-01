use std::{fmt::Display, ops::Add, time::Instant};

use lazy_regex::regex;

use super::InvalidDuration;

type StdDuration = std::time::Duration;

#[derive(Clone, PartialEq, Eq, PartialOrd, Copy)]
pub struct Duration {
    inner: StdDuration,
}

pub struct DurationRange {
    min: Duration,
    max: Duration,
}

impl Duration {
    pub const fn new(secs: u64) -> Self {
        Self {
            inner: std::time::Duration::from_secs(secs),
        }
    }
}

impl DurationRange {
    pub const fn new(min: u64, max: u64) -> Self {
        Self {
            min: Duration::new(min),
            max: Duration::new(max),
        }
    }

    pub(crate) fn contains(&self, duration: Duration) -> Result<Duration, InvalidDuration> {
        if self.min <= duration && duration <= self.max {
            Ok(duration)
        } else {
            Err(InvalidDuration::DurationMustLieBetween {
                min: self.min.to_string(),
                max: self.max.to_string(),
            })
        }
    }
}

impl Default for Duration {
    fn default() -> Self {
        Self {
            inner: StdDuration::from_secs(60),
        }
    }
}

impl Add<Instant> for Duration {
    type Output = Instant;

    fn add(self, rhs: Instant) -> Self::Output {
        rhs + self.inner
    }
}

impl From<Duration> for StdDuration {
    fn from(duration: Duration) -> Self {
        duration.inner
    }
}

impl From<&Duration> for StdDuration {
    fn from(duration: &Duration) -> Self {
        duration.inner
    }
}
impl TryFrom<&str> for Duration {
    type Error = InvalidDuration;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let splitter = regex!(r"^(\d+)(h|min|s|ms|μs|ns)$");

        splitter
            .captures(value)
            .map_or(Err(InvalidDuration::InvalidSyntax), |group| {
                let value = group[1].parse::<u64>()?;
                if value == 0 {
                    Ok(std::time::Duration::ZERO)
                } else {
                    match &group[2] {
                        "h" => Ok(std::time::Duration::from_secs(value * 3_600)),
                        "min" => Ok(std::time::Duration::from_secs(value * 60)),
                        "s" => Ok(std::time::Duration::from_secs(value)),
                        "ms" => Ok(std::time::Duration::from_millis(value)),
                        "μs" => Ok(std::time::Duration::from_micros(value)),
                        "ns" => Ok(std::time::Duration::from_nanos(value)),

                        sym => Err(InvalidDuration::UnsupportedDurationSymbol {
                            sym: sym.to_string(),
                        }),
                    }
                }
            })
            .map(|inner| Self { inner })
    }
}

impl From<(u64, u64)> for DurationRange {
    fn from((min, max): (u64, u64)) -> Self {
        DurationRange::new(min, max)
    }
}

impl From<Duration> for clap::builder::OsStr {
    fn from(duration: Duration) -> Self {
        (format!("{}", duration)).into()
    }
}

impl Display for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let secs = self.inner.as_secs();
        f.write_str(
            match secs {
                _ if secs < 60 || secs % 60 != 0 => format!("{}s", secs),
                _ if secs < 90 || secs % 3_600 != 0 => format!("{}min", secs / 60),
                _ => format!("{}h", secs / 3_600),
            }
            .as_str(),
        )
    }
}
