use std::{fmt::Display, ops::Add, time::Instant};

use lazy_regex::regex;
use syn::{
    parse::{Parse, ParseStream},
    LitInt,
};

use crate::{DurationError, DurationHumanValidator};

type StdDuration = std::time::Duration;

pub const MICRO_SEC: u64 = 1_000;
pub const MILLI_SEC: u64 = 1_000 * MICRO_SEC;
pub(crate) const SEC: u64 = 1_000 * MILLI_SEC;
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
    pub const ONE_SECOND: Self = Self::new(SEC);
    pub const ONE_MILLISECOND: Self = Self::new(MILLI_SEC);

    #[must_use]
    pub const fn new(nanos: u64) -> Self {
        Self {
            inner: std::time::Duration::from_nanos(nanos),
        }
    }

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
            inner: StdDuration::from_millis(MINUTE),
        }
    }
}

impl Parse for DurationHuman {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let duration_with_unit = input.parse::<LitInt>()?.to_string();
        match TryInto::<Self>::try_into(duration_with_unit.as_str()) {
            Ok(duration_human) => Ok(duration_human),
            Err(e) => Err(input.error(e.to_string())),
        }
    }
}

impl Add<Instant> for DurationHuman {
    type Output = Instant;

    /// Create a new std::time::Instant by adding one to this duration
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
    /// For non human interaction features, just unwrap the std::time::Duration
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
    fn from(ms: u64) -> Self {
        Self::new(ms)
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
                    let unit = group
                        .get(2)
                        .or_else(|| group.get(3).or_else(|| group.get(4)))
                        .unwrap();

                    match unit.as_str() {
                        "centuries" => Ok(value * CENTURY),
                        "century" => Ok(value * CENTURY),
                        "year" => Ok(value * YEAR),
                        "month" => Ok(value * MONTH),
                        "week" => Ok(value * WEEK),
                        "day" => Ok(value * DAY),
                        "h" => Ok(value * HOUR),
                        "min" => Ok(value * MINUTE),
                        "s" => Ok(value * SEC),
                        "ms" => Ok(value * MILLI_SEC),
                        "μs" => Ok(value * MICRO_SEC),
                        "ns" => Ok(value),

                        sym => Err(DurationError::UnsupportedSymbol {
                            sym: sym.to_string(),
                        }),
                    }
                }
            })
            .fold(0, |ms, r| {
                let d = r.map_or_else(
                    |err| {
                        unexpected = Some(err);
                        0
                    },
                    |d| d,
                );
                ms + d
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

impl Display for DurationHuman {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut nanos: u64 = self.into();
        if f.alternate() {
            let durations: Vec<String> = [
                (CENTURY, " century", " centuries"),
                (YEAR, " year", " years"),
                (MONTH, " month", " months"),
                (WEEK, " week", " weeks"),
                (DAY, " day", " days"),
                (HOUR, "h", "h"),
                (MINUTE, "min", "min"),
                (SEC, "s", "s"),
                (MILLI_SEC, "ms", "ms"),
                (MICRO_SEC, "μs", "μs"),
                (1, "ns", "ns"),
            ]
            .iter()
            .filter_map(|(part_ms, unit_singular, unit_plural)| {
                let part = nanos / part_ms;
                nanos %= part_ms;
                if part > 0 {
                    Some(format!(
                        "{}{}",
                        part,
                        if part > 1 { unit_plural } else { unit_singular }
                    ))
                } else {
                    None
                }
            })
            .collect();
            f.write_str(durations.join(" ").as_str())
        } else {
            f.write_str(
                match nanos {
                    _ if nanos < MICRO_SEC || nanos % MICRO_SEC != 0 => {
                        format!("{}ns", nanos)
                    }
                    _ if nanos < MILLI_SEC || nanos % MILLI_SEC != 0 => {
                        format!("{}μs", nanos / MICRO_SEC)
                    }
                    _ if nanos < SEC || nanos % SEC != 0 => format!("{}ms", nanos / MILLI_SEC),
                    _ if nanos < MINUTE || nanos % MINUTE != 0 => format!("{}s", nanos / SEC),
                    _ if nanos < HOUR || nanos % HOUR != 0 => format!("{}min", nanos / MINUTE),
                    _ if nanos < DAY || nanos % DAY != 0 => format!("{}h", nanos / HOUR),
                    _ if nanos < WEEK || nanos % WEEK != 0 => {
                        format!(
                            "{} day{}",
                            nanos / DAY,
                            if nanos / DAY > 1 { "s" } else { "" }
                        )
                    }
                    _ if nanos < MONTH || nanos % MONTH != 0 => {
                        format!(
                            "{} week{}",
                            nanos / WEEK,
                            if nanos / WEEK > 1 { "s" } else { "" }
                        )
                    }
                    _ if nanos < YEAR || nanos % YEAR != 0 => format!(
                        "{} month{}",
                        nanos / MONTH,
                        if nanos / YEAR > 1 { "s" } else { "" }
                    ),
                    _ if nanos < CENTURY || nanos % CENTURY != 0 => {
                        format!(
                            "{} year{}",
                            nanos / YEAR,
                            if nanos / YEAR > 1 { "s" } else { "" }
                        )
                    }
                    _ => format!(
                        "{} centur{}",
                        nanos / CENTURY,
                        if nanos / CENTURY > 1 { "ies" } else { "y" }
                    ),
                }
                .as_str(),
            )
        }
    }
}

#[test]
fn one_sec() {
    let duration = DurationHuman::try_from("1s").unwrap();

    dbg!(format!("{}", duration));
    dbg!(format!("{:#}", duration));
}

#[test]
fn roundtrip() {
    let duration = format!(
        "{:#}",
        DurationHuman::try_from("It will take 2years 1 week 3days 5h 6min and 10s.").unwrap()
    );
    assert_eq!(duration, format!("2 years 1 week 3 days 5h 6min 10s"));

    let duration = format!("{:#}", DurationHuman::try_from(duration.as_str()).unwrap());
    assert_eq!(duration, format!("2 years 1 week 3 days 5h 6min 10s"));
}

#[test]
fn beware_of_unrecognised() -> Result<(), DurationError> {
    let duration = DurationHuman::try_from("It will take 2years 1 week 3 dagen 5h 6min and 10s.")?;
    assert_eq!(
        format!("{:#}", duration),
        format!("2 years 1 week 5h 6min 10s")
    );
    Ok(())
}

#[test]
fn max() -> Result<(), DurationError> {
    let duration = DurationHuman::try_from(
        "5 centuries 84 years 11 months 1 week 6 days 23h 34min 33s 709ms 551μs 615ns",
    )?;
    let pretty = format!("{:#}", duration);
    let duration_from_pretty = DurationHuman::try_from(pretty.as_str())?;
    assert_eq!(duration, duration_from_pretty);
    Ok(())
}
