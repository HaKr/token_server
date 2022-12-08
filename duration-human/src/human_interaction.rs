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
/// assert_eq!(format!("{:#}", duration), "5 centuries 84 years 11 months 1 week 6 days 23h 34min 33s 709ms 551μs 615ns".to_string());
/// // roundtrip
/// let duration = DurationHuman::try_from("5 centuries 84 years 11 months 1 week 6 days 23h 34min 33s 709ms 551μs 615ns").unwrap();
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
    pub const MONTH: u64 = 30 * Self::DAY;
    pub const YEAR: u64 = 365 * Self::DAY;
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
    fn from(nanos: u64) -> Self {
        Self::new(nanos)
    }
}

impl TryFrom<&str> for DurationHuman {
    type Error = DurationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let splitter = regex!(
            r"(\d+)\s*(?:(century|centuries)|(year|month|week|day)(?:s?)|(h|min|s|ms|μs|ns))"
        );

        if !splitter.is_match(value) {
            return Err(DurationError::InvalidSyntax);
        }
        let mut unexpected = None;
        let nanos = splitter
            .captures_iter(value)
            .map(|group| {
                let value = group[1].parse::<u64>()?;
                if value == 0 {
                    Ok(0)
                } else {
                    #[allow(clippy::unwrap_used)] // somehow the RE has four groups
                    let unit = group
                        .get(2)
                        .or_else(|| group.get(3).or_else(|| group.get(4)))
                        .unwrap();

                    match unit.as_str() {
                        "century" | "centuries" => Ok(value * Self::CENTURY),
                        "year" => Ok(value * Self::YEAR),
                        "month" => Ok(value * Self::MONTH),
                        "week" => Ok(value * Self::WEEK),
                        "day" => Ok(value * Self::DAY),
                        "h" => Ok(value * Self::HOUR),
                        "min" => Ok(value * Self::MINUTE),
                        "s" => Ok(value * Self::SEC),
                        "ms" => Ok(value * Self::MILLI_SEC),
                        "μs" => Ok(value * Self::MICRO_SEC),
                        "ns" => Ok(value),

                        sym => Err(DurationError::UnsupportedSymbol {
                            sym: sym.to_string(),
                        }),
                    }
                }
            })
            .fold(0, |nanos, r| {
                let d = r.map_or_else(
                    |err| {
                        unexpected = Some(err);
                        0
                    },
                    |d| d,
                );
                nanos + d
            });

        unexpected.map_or(
            Ok(Self {
                inner: std::time::Duration::from_nanos(nanos),
            }),
            Err,
        )
    }
}

impl From<DurationHuman> for clap::builder::OsStr {
    fn from(duration: DurationHuman) -> Self {
        duration.to_string().into()
    }
}

impl From<&DurationHuman> for u64 {
    #[allow(clippy::cast_possible_truncation)] // cast is okay, as u64::MAX as milliseconds is more than 500 million years
    fn from(duration: &DurationHuman) -> Self {
        duration.inner.as_nanos() as Self
    }
}
