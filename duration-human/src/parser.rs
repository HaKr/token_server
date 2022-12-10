use std::{ops::Add, time::Instant};

use lazy_regex::regex;

use crate::{DurationError, DurationHumanValidator};

type StdDuration = std::time::Duration;

/// Define a Duration in human readable form
///
/// ## Examples
/// ```
/// # use duration_human::{DurationHuman, DurationError};
/// let duration = DurationHuman::try_from("80h").unwrap();
/// assert_eq!(format!("{:#}", duration), "3 days 8h".to_string());
/// assert_eq!(format!("{}", duration), "80h".to_string());
/// let duration = DurationHuman::try_from("72h").unwrap();
/// assert_eq!(format!("{:#}", duration), "3 days".to_string());
/// assert_eq!(format!("{}", duration), "3 days".to_string());
/// let duration = DurationHuman::try_from("18446744073709551615ns").unwrap();
/// assert_eq!(format!("{:#}", duration), "5 centuries 84 years 6 months 2 weeks 1 day 8h 34min 33s 709ms 551μs 615ns".to_string());
/// // roundtrip
/// let duration = DurationHuman::try_from("5 centuries 84 years 6 months 2 weeks 1 day 8h 34min 33s 709ms 551μs 615ns").unwrap();
/// let pretty = format!("{:#}", duration);
/// let duration_from_pretty = DurationHuman::try_from(pretty.as_str())?;
/// assert_eq!(duration, duration_from_pretty);
/// // precision is nano second
/// let duration = DurationHuman::try_from("604800μs").unwrap();
/// assert_eq!(format!("{:#}", duration), "604ms 800μs".to_string());
/// assert_eq!(duration.to_string(), "604800μs".to_string());
/// let duration = DurationHuman::try_from("604800ms").unwrap();
/// assert_eq!(format!("{:#}", duration), "10min 4s 800ms".to_string());
/// assert_eq!(duration.to_string(), "604800ms".to_string());
/// let duration = DurationHuman::try_from("604800s").unwrap();
/// assert_eq!(format!("{:#}", duration), "1 week".to_string());
/// let duration = DurationHuman::try_from("604800s").unwrap();
/// assert_eq!(format!("{:#}", duration), "1 week".to_string());
/// assert_eq!(format!("{}", duration), "1 week".to_string());
/// let duration = DurationHuman::try_from("608430s").unwrap();
/// assert_eq!(format!("{:#}", duration), "1 week 1h 30s".to_string());
/// assert_eq!(format!("{}", duration), "608430s".to_string());
/// # Ok::<(), DurationError>(())
/// ```
#[derive(Clone, PartialEq, Eq, PartialOrd, Copy, Debug)]
pub struct DurationHuman {
    inner: StdDuration,
}

impl DurationHuman {
    pub const MICRO_SEC: u64 = 1_000;
    pub const MILLI_SEC: u64 = 1_000 * Self::MICRO_SEC;
    pub const SEC: u64 = 1_000 * Self::MILLI_SEC;
    pub const MINUTE: u64 = 60 * Self::SEC;
    pub const HOUR: u64 = 60 * Self::MINUTE;
    pub const DAY: u64 = 24 * Self::HOUR;
    pub const WEEK: u64 = 7 * Self::DAY;
    pub const YEAR: u64 = 31_557_600 * Self::SEC;
    pub const MONTH: u64 = Self::YEAR / 12;
    pub const CENTURY: u64 = 100 * Self::YEAR;

    pub const ONE_SECOND: Self = Self::new(Self::SEC);
    pub const ONE_MILLISECOND: Self = Self::new(Self::MILLI_SEC);

    #[must_use]
    pub const fn new(nanos: u64) -> Self {
        Self {
            inner: std::time::Duration::from_nanos(nanos),
        }
    }

    /// Create a new duration from a human redable string
    ///
    /// ## Errors
    /// `DurationError` when the parsing fails
    pub fn parse(human_readable: &str) -> Result<Self, DurationError> {
        Self::try_from(human_readable)
    }

    #[must_use]
    pub fn is_in(&self, range: &DurationHumanValidator) -> bool {
        range.contains(self)
    }
}

impl Default for DurationHuman {
    /// Defaults to a 1min duration
    fn default() -> Self {
        Self {
            inner: StdDuration::from_millis(Self::MINUTE),
        }
    }
}

impl Add<Instant> for DurationHuman {
    type Output = Instant;

    /// Create a new `std::time::Instant` by adding one to this duration
    ///
    /// ## Example
    /// ```
    /// # use std::time::Instant;
    /// # use duration_human::{DurationHuman, DurationError};
    /// let instant = Instant::now();
    /// let duration = DurationHuman::try_from("420s")?;
    /// let after = duration + instant;
    /// let diff = DurationHuman::from(after - instant);
    /// assert_eq!(format!("{}", diff), format!("7min"));
    /// # Ok::<(),DurationError>(())
    /// ```
    fn add(self, rhs: Instant) -> Self::Output {
        rhs + self.inner
    }
}

impl From<StdDuration> for DurationHuman {
    fn from(inner: StdDuration) -> Self {
        Self { inner }
    }
}

impl From<&DurationHuman> for StdDuration {
    /// For non human interaction features, just unwrap the `std::time::Duration`
    ///
    /// ## Example
    /// ```
    /// # use duration_human::{DurationHuman, DurationError};
    /// let duration = DurationHuman::try_from("5min")?;
    /// let duration = std::time::Duration::from(&duration);
    /// assert_eq!(duration.as_secs_f32(), 300_f32);
    /// # Ok::<(),DurationError>(())
    /// ```
    fn from(duration: &DurationHuman) -> Self {
        duration.inner
    }
}

impl From<u64> for DurationHuman {
    /// Create a duration in nano seconds
    fn from(nanos: u64) -> Self {
        Self::new(nanos)
    }
}

impl TryFrom<&str> for DurationHuman {
    type Error = DurationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let matcher = regex!(
            r"^(?:(\d+)\s*(?:(century|centuries)|(year|month|week|day)(?:s?)|(h|min|s|ms|μs|ns))\s*)*$"
        );

        let splitter = regex!(
            r"(\d+)\s*(?:(century|centuries)|(year|month|week|day)(?:s?)|(h|min|s|ms|μs|ns))"
        );

        if !matcher.is_match(value) {
            return Err(DurationError::InvalidSyntax);
        }

        splitter
            .captures_iter(value)
            .map(|group| {
                let value = group[1].parse::<u64>()?;

                if value == 0 {
                    Ok(DurationPart::default())
                } else {
                    let part: &str = group[0].as_ref();

                    #[allow(clippy::unwrap_used)] // somehow the RE has four groups
                    let unit = group
                        .get(2)
                        .or_else(|| group.get(3).or_else(|| group.get(4)))
                        .unwrap();

                    match unit.as_str() {
                        "century" | "centuries" => (part, value, Self::CENTURY).try_into(),
                        "year" => (part, value, Self::YEAR).try_into(),
                        "month" => (part, value, Self::MONTH).try_into(),
                        "week" => (part, value, Self::WEEK).try_into(),
                        "day" => (part, value, Self::DAY).try_into(),
                        "h" => (part, value, Self::HOUR).try_into(),
                        "min" => (part, value, Self::MINUTE).try_into(),
                        "s" => (part, value, Self::SEC).try_into(),
                        "ms" => (part, value, Self::MILLI_SEC).try_into(),
                        "μs" => (part, value, Self::MICRO_SEC).try_into(),
                        "ns" => (part, value, 1).try_into(),
                        sym => Err(DurationError::UnitMatchAndRegexNotInSync {
                            sym: sym.to_string(),
                        }),
                    }
                }
            })
            .fold(Ok(0), |nanos_sum, part| {
                nanos_sum.and_then(|nanos_sum| {
                    part.and_then(|duration_part| duration_part.add(nanos_sum))
                })
            })
            .map(Self::from)
    }
}

impl From<DurationHuman> for clap::builder::OsStr {
    fn from(duration: DurationHuman) -> Self {
        duration.to_string().into()
    }
}

impl From<&DurationHuman> for u64 {
    /// convert this duration into nano seconds
    #[allow(clippy::cast_possible_truncation)] // cast is okay, as u64::MAX as milliseconds is more than 500 million years
    fn from(duration: &DurationHuman) -> Self {
        duration.inner.as_nanos() as Self
    }
}

#[derive(Default)]
struct DurationPart {
    part: String,
    nanos: u64,
}

impl TryFrom<(&str, u64, u64)> for DurationPart {
    type Error = DurationError;

    /// Create a `DurationPart` from a value and multiplication factor (both u64)
    ///
    /// ## Errors
    /// if the product would overflow 2^64, the return is `DurationError::IntegerOverflowAt`
    fn try_from((part, value, factor): (&str, u64, u64)) -> Result<Self, Self::Error> {
        if factor < 1 {
            return Ok(Self::default());
        }

        if value > u64::MAX / factor {
            return Err(DurationError::IntegerOverflowAt {
                duration: part.to_string(),
            });
        }

        Ok(Self {
            part: part.to_string(),
            nanos: value * factor,
        })
    }
}

impl DurationPart {
    /// Add another nano second value
    ///
    /// ## Errors
    /// if the sum would overflow 2^64, the return is `DurationError::IntegerOverflowAt`
    fn add(&self, rhs: u64) -> Result<u64, DurationError> {
        if self.nanos > u64::MAX - rhs {
            return Err(DurationError::IntegerOverflowAt {
                duration: self.part.to_string(),
            });
        }

        Ok(self.nanos + rhs)
    }
}
