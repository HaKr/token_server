use lazy_regex::regex;
use std::{fmt::Display, num::ParseIntError, ops::Add, time::Instant};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, LitInt, Token,
};
use thiserror::Error;

type StdDuration = std::time::Duration;

const SEC: u64 = 1_000;
const MINUTE: u64 = 60 * SEC;
const HOUR: u64 = 60 * MINUTE;
const DAY: u64 = 24 * HOUR;

#[derive(Clone, PartialEq, Eq, PartialOrd, Copy)]
pub struct DurationInms {
    inner: StdDuration,
}

#[derive(Default)]
pub struct DurationInmsRangeAndDefault {
    pub min: DurationInms,
    pub default: DurationInms,
    pub max: DurationInms,
}

#[derive(Error, Debug)]
pub enum InvalidDuration {
    #[error("Duration must be specified as a positive integer, immediately followed by days, h, min, s, ms, μs or ns")]
    InvalidSyntax,

    #[error("Invalid duration value")]
    InvalidValue {
        #[from]
        source: ParseIntError,
    },

    #[error("'{sym}' is not supported as a duration symbol")]
    UnsupportedDurationSymbol { sym: String },

    #[error("Duration must lie between {range}")]
    DurationMustLieBetween { range: String },
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
    pub fn is_in(&self, range: &DurationInmsRangeAndDefault) -> bool {
        range.contains(self)
    }

    /// # Errors
    ///
    /// Will return `Err` if self is not within the given range
    /// permission to read it.
    pub fn must_be_in(self, range: &DurationInmsRangeAndDefault) -> Result<Self, InvalidDuration> {
        if range.contains(&self) {
            Ok(self)
        } else {
            Err(InvalidDuration::DurationMustLieBetween {
                range: range.to_string(),
            })
        }
    }
}

impl DurationInmsRangeAndDefault {
    #[must_use]
    pub const fn new(minimal_seconds: u64, default_seconds: u64, maximal_seconds: u64) -> Self {
        Self {
            min: DurationInms::new(minimal_seconds),
            default: DurationInms::new(default_seconds),
            max: DurationInms::new(maximal_seconds),
        }
    }

    #[must_use]
    pub fn contains(&self, duration: &DurationInms) -> bool {
        self.min <= *duration && *duration <= self.max
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

impl Parse for DurationInmsRangeAndDefault {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let parser = Punctuated::<ParsedDuration, Token![,]>::parse_separated_nonempty;
        let durations: Vec<ParsedDuration> = parser(input)?.into_iter().collect();
        let result = durations.try_into();
        match result {
            Ok(result) => Ok(result),
            Err(err) => Err(input.error(err)),
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

impl TryFrom<&str> for DurationInms {
    type Error = InvalidDuration;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let splitter = regex!(r"^(\d+)(day|h|min|s|ms|μs|ns)$");

        splitter
            .captures(value)
            .map_or(Err(InvalidDuration::InvalidSyntax), |group| {
                let value = group[1].parse::<u64>()?;
                if value == 0 {
                    Ok(std::time::Duration::ZERO)
                } else {
                    match &group[2] {
                        "day" => Ok(std::time::Duration::from_millis(value * DAY)),
                        "h" => Ok(std::time::Duration::from_millis(value * HOUR)),
                        "min" => Ok(std::time::Duration::from_millis(value * MINUTE)),
                        "s" => Ok(std::time::Duration::from_millis(value * SEC)),
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

impl From<&DurationInmsRangeAndDefault> for (u64, u64, u64) {
    fn from(duration: &DurationInmsRangeAndDefault) -> Self {
        (
            (&duration.min).into(),
            (&duration.default).into(),
            (&duration.max).into(),
        )
    }
}

impl From<&DurationInmsRangeAndDefault> for (String, String, String) {
    fn from(duration: &DurationInmsRangeAndDefault) -> Self {
        (
            duration.min.to_string(),
            duration.default.to_string(),
            duration.max.to_string(),
        )
    }
}

impl Display for DurationInmsRangeAndDefault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{min} and {max}",
            min = self.min,
            max = self.max
        ))
    }
}

impl Display for DurationInms {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ms: u64 = self.into();
        f.write_str(
            match ms {
                _ if ms < SEC || ms % SEC != 0 => format!("{}ms", ms),
                _ if ms < MINUTE || ms % MINUTE != 0 => format!("{}s", ms / SEC),
                _ if ms < HOUR || ms % HOUR != 0 => format!("{}min", ms / MINUTE),
                _ if ms < DAY || ms % DAY != 0 => format!("{}h", ms / HOUR),
                _ => format!("{} days", ms / DAY),
            }
            .as_str(),
        )
    }
}

struct ParsedDuration {
    arg: DurationRangeArgument,
    duration: DurationInms,
}

enum DurationRangeArgument {
    Min,
    Default,
    Max,
}

impl Parse for DurationRangeArgument {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let name = name.to_string();
        match name.as_str() {
            "min" => Ok(Self::Min),
            "max" => Ok(Self::Max),
            "default" => Ok(Self::Default),

            _ => Err(input.error(format!(
                "duration argument '{}' not recognized, use min, max and default",
                name
            ))),
        }
    }
}

impl Parse for ParsedDuration {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: DurationRangeArgument = input.parse()?;
        let _punc: Token![:] = input.parse()?;
        let value: DurationInms = input.parse()?;

        Ok(Self {
            arg: name,
            duration: value,
        })
    }
}

#[derive(Default)]
struct DurationRangeWithOptionalValues {
    min: Option<DurationInms>,
    max: Option<DurationInms>,
    default: Option<DurationInms>,
}

impl From<Vec<ParsedDuration>> for DurationRangeWithOptionalValues {
    fn from(value: Vec<ParsedDuration>) -> Self {
        value.iter().fold(Self::default(), |mut y, x| {
            match x.arg {
                DurationRangeArgument::Min => y.min = Some(x.duration),
                DurationRangeArgument::Max => y.max = Some(x.duration),
                DurationRangeArgument::Default => y.default = Some(x.duration),
            }

            y
        })
    }
}

impl TryFrom<Vec<ParsedDuration>> for DurationInmsRangeAndDefault {
    type Error = String;

    fn try_from(value: Vec<ParsedDuration>) -> Result<Self, Self::Error> {
        let optionals: DurationRangeWithOptionalValues = value.into();
        optionals.min.map_or_else(
            || Err("could not find min duration".to_string()),
            |min| {
                optionals.max.map_or_else(
                    || Err("could not find max duration".to_string()),
                    |max| {
                        optionals.default.map_or_else(
                            || Err("could not find default duration".to_string()),
                            |default| Ok(Self { min, default, max }),
                        )
                    },
                )
            },
        )
    }
}
