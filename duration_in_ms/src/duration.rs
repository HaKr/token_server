use lazy_regex::regex;
use std::{fmt::Display, ops::Add, time::Instant};
use syn::{
    parse::{Parse, ParseStream},
    LitInt,
};

use crate::{DurationError, DurationInmsValidator};

type StdDuration = std::time::Duration;

pub(crate) const SEC: u64 = 1_000;
const MINUTE: u64 = 60 * SEC;
const HOUR: u64 = 60 * MINUTE;
const DAY: u64 = 24 * HOUR;
const WEEK: u64 = 7 * DAY;
const MONTH: u64 = 30 * DAY;
const YEAR: u64 = 365 * DAY;
const CENTURY: u64 = 100 * YEAR;

/// Define a Duration in human readable form
///
/// ## Examples
/// ```
/// # use duration_in_ms::DurationInms;
/// let duration = DurationInms::try_from("80h").unwrap();
/// assert_eq!(format!("{:#}", duration), "3 day 8h".to_string());
/// assert_eq!(format!("{}", duration), "80h".to_string());
/// let duration = DurationInms::try_from("72h").unwrap();
/// assert_eq!(format!("{:#}", duration), "3 day".to_string());
/// assert_eq!(format!("{}", duration), "3 day".to_string());
/// let duration = DurationInms::try_from("18446744073709551615ns").unwrap();
/// assert_eq!(format!("{:#}", duration), "5 century 84 year 11 month 1 week 6 day 23h 34min 33s 709ms".to_string());
/// let duration = DurationInms::try_from("18446744073709551615ms").unwrap();
/// assert_eq!(format!("{:#}", duration), "5849424 century 17 year 4 month 1 week 2 day 14h 25min 51s 615ms".to_string());
/// // precision is ms
/// let duration = DurationInms::try_from("604800μs").unwrap();
/// assert_eq!(format!("{:#}", duration), "604ms".to_string());
/// assert_eq!(duration.to_string(), "604ms".to_string());
/// let duration = DurationInms::try_from("604800ms").unwrap();
/// assert_eq!(format!("{:#}", duration), "10min 4s 800ms".to_string());
/// assert_eq!(duration.to_string(), "604800ms".to_string());
/// let duration = DurationInms::try_from("604800s").unwrap();
/// assert_eq!(format!("{:#}", duration), "1 week".to_string());
/// let duration = DurationInms::try_from("604800s").unwrap();
/// assert_eq!(format!("{:#}", duration), "1 week".to_string(),"A");
/// assert_eq!(format!("{}", duration), "1 week".to_string(),"B");
/// let duration = DurationInms::try_from("608430s").unwrap();
/// assert_eq!(format!("{:#}", duration), "1 week 1h 30s".to_string());
/// assert_eq!(format!("{}", duration), "608430s".to_string(),"C");
/// ```
#[derive(Clone, PartialEq, Eq, PartialOrd, Copy)]
pub struct DurationInms {
    inner: StdDuration,
}

impl DurationInms {
    pub const ONE_SECOND: Self = Self::new(SEC);
    pub const ONE_MILLISECOND: Self = Self::new(1);

    #[must_use]
    pub const fn new(ms: u64) -> Self {
        Self {
            inner: std::time::Duration::from_millis(ms),
        }
    }

    #[must_use]
    pub fn is_in(&self, range: &DurationInmsValidator) -> bool {
        range.contains(self)
    }
}

impl Default for DurationInms {
    fn default() -> Self {
        Self {
            inner: StdDuration::from_millis(MINUTE),
        }
    }
}

impl Parse for DurationInms {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let duration_with_unit = input.parse::<LitInt>()?.to_string();
        match TryInto::<Self>::try_into(duration_with_unit.as_str()) {
            Ok(duration_in_ms) => Ok(duration_in_ms),
            Err(e) => Err(input.error(e.to_string())),
        }
    }
}

impl Add<Instant> for DurationInms {
    type Output = Instant;

    fn add(self, rhs: Instant) -> Self::Output {
        rhs + self.inner
    }
}

impl From<&DurationInms> for StdDuration {
    fn from(duration: &DurationInms) -> Self {
        duration.inner
    }
}

impl From<u64> for DurationInms {
    fn from(ms: u64) -> Self {
        Self::new(ms)
    }
}

impl TryFrom<&str> for DurationInms {
    type Error = DurationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let splitter = regex!(r"^(\d+)\s*(year|month|week|day|h|min|s|ms|μs|ns)$");

        splitter
            .captures(value)
            .map_or(Err(DurationError::InvalidSyntax), |group| {
                let value = group[1].parse::<u64>()?;
                if value == 0 {
                    Ok(std::time::Duration::ZERO)
                } else {
                    match &group[2] {
                        "year" => Ok(std::time::Duration::from_millis(value * YEAR)),
                        "month" => Ok(std::time::Duration::from_millis(value * MONTH)),
                        "week" => Ok(std::time::Duration::from_millis(value * WEEK)),
                        "day" => Ok(std::time::Duration::from_millis(value * DAY)),
                        "h" => Ok(std::time::Duration::from_millis(value * HOUR)),
                        "min" => Ok(std::time::Duration::from_millis(value * MINUTE)),
                        "s" => Ok(std::time::Duration::from_millis(value * SEC)),
                        "ms" => Ok(std::time::Duration::from_millis(value)),
                        "μs" => Ok(std::time::Duration::from_micros(value)),
                        "ns" => Ok(std::time::Duration::from_nanos(value)),

                        sym => Err(DurationError::UnsupportedSymbol {
                            sym: sym.to_string(),
                        }),
                    }
                }
            })
            .map(|inner| Self { inner })
    }
}

impl From<DurationInms> for clap::builder::OsStr {
    fn from(duration: DurationInms) -> Self {
        duration.to_string().into()
    }
}

impl From<&DurationInms> for u64 {
    #[allow(clippy::cast_possible_truncation)] // cast is okay, as u64::MAX as milliseconds is more than 500 million years
    fn from(duration: &DurationInms) -> Self {
        duration.inner.as_millis() as Self
    }
}

impl Display for DurationInms {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ms: u64 = self.into();
        if f.alternate() {
            let durations: Vec<String> = [
                (CENTURY, " century"),
                (YEAR, " year"),
                (MONTH, " month"),
                (WEEK, " week"),
                (DAY, " day"),
                (HOUR, "h"),
                (MINUTE, "min"),
                (SEC, "s"),
                (1, "ms"),
            ]
            .iter()
            .filter_map(|(part_ms, unit)| {
                let part = ms / part_ms;
                ms %= part_ms;
                if part > 0 {
                    Some(format!("{}{}", part, unit))
                } else {
                    None
                }
            })
            .collect();
            f.write_str(durations.join(" ").as_str())
        } else {
            f.write_str(
                match ms {
                    _ if ms < SEC || ms % SEC != 0 => format!("{}ms", ms),
                    _ if ms < MINUTE || ms % MINUTE != 0 => format!("{}s", ms / SEC),
                    _ if ms < HOUR || ms % HOUR != 0 => format!("{}min", ms / MINUTE),
                    _ if ms < DAY || ms % DAY != 0 => format!("{}h", ms / HOUR),
                    _ if ms < WEEK || ms % WEEK != 0 => format!("{} day", ms / DAY),
                    _ if ms < MONTH || ms % MONTH != 0 => format!("{} week", ms / WEEK),
                    _ if ms < YEAR || ms % YEAR != 0 => format!("{} month", ms / MONTH),
                    _ if ms < CENTURY || ms % CENTURY != 0 => format!("{} year", ms / YEAR),
                    _ => format!("{} century", ms / CENTURY),
                }
                .as_str(),
            )
        }
    }
}
