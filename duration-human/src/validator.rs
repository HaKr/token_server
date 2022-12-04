use std::fmt::Display;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, Token,
};

use crate::{DurationError, DurationHuman, SEC};

#[derive(Default)]
pub struct DurationHumanValidator {
    pub min: DurationHuman,
    pub default: DurationHuman,
    pub max: DurationHuman,
}

impl DurationHumanValidator {
    #[must_use]
    pub const fn new(minimal_ms: u64, default_ms: u64, maximal_ms: u64) -> Self {
        assert!(minimal_ms <= default_ms && default_ms <= maximal_ms);
        assert!(minimal_ms >= SEC);
        assert!(default_ms >= SEC);
        assert!(maximal_ms >= SEC);

        Self {
            min: DurationHuman::new(minimal_ms),
            default: DurationHuman::new(default_ms),
            max: DurationHuman::new(maximal_ms),
        }
    }

    const fn try_new(
        minimal_ms: u64,
        default_ms: u64,
        maximal_ms: u64,
    ) -> Result<Self, DurationError> {
        if minimal_ms < SEC {
            Err(DurationError::DurationValidationMinMustBeMoreThanOneSecond)
        } else if default_ms < SEC {
            Err(DurationError::DurationValidationDefaultMustBeMoreThanOneSecond)
        } else {
            Ok(Self {
                min: DurationHuman::new(minimal_ms),
                default: DurationHuman::new(default_ms),
                max: DurationHuman::new(maximal_ms),
            })
        }
    }

    /// To be used as a validate_parser for clap
    ///
    /// ```compile_error
    ///  validate_parser = {|lifetime: &str|duration_range.parse_and_validate(lifetime)}
    /// ```
    /// # Errors
    ///
    /// Will return `Err` if duration is not within the given range
    /// permission to read it.
    pub fn parse_and_validate(&self, duration: &str) -> Result<DurationHuman, DurationError> {
        let duration_in_ms = DurationHuman::try_from(duration)?;

        if self.contains(&duration_in_ms) {
            Ok(duration_in_ms)
        } else {
            Err(DurationError::DurationMustLieBetween {
                range: self.to_string(),
            })
        }
    }

    #[must_use]
    pub fn contains(&self, duration: &DurationHuman) -> bool {
        self.min <= *duration && *duration <= self.max
    }
}

impl Parse for DurationHumanValidator {
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

impl From<&DurationHumanValidator> for (u64, u64, u64) {
    fn from(duration: &DurationHumanValidator) -> Self {
        (
            (&duration.min).into(),
            (&duration.default).into(),
            (&duration.max).into(),
        )
    }
}

impl From<&DurationHumanValidator> for (String, String, String) {
    fn from(duration: &DurationHumanValidator) -> Self {
        (
            duration.min.to_string(),
            duration.default.to_string(),
            duration.max.to_string(),
        )
    }
}

impl Display for DurationHumanValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{min} and {max}",
            min = self.min,
            max = self.max
        ))
    }
}

struct ParsedDuration {
    arg: DurationRangeArgument,
    duration: DurationHuman,
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
        let value: DurationHuman = input.parse()?;

        Ok(Self {
            arg: name,
            duration: value,
        })
    }
}

#[derive(Default)]
struct DurationRangeWithOptionalValues {
    min: Option<DurationHuman>,
    max: Option<DurationHuman>,
    default: Option<DurationHuman>,
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

impl TryFrom<(DurationHuman, DurationHuman)> for DurationHumanValidator {
    type Error = DurationError;

    fn try_from(value: (DurationHuman, DurationHuman)) -> Result<Self, Self::Error> {
        let (minimal, maximal) = &value;
        if minimal > maximal {
            Err(DurationError::DurationValidationMinMustBeLessOrEqualMax {
                minimal: minimal.to_string(),
                maximal: maximal.to_string(),
            })
        } else {
            let (minimal_ms, maximal_ms): (u64, u64) = (minimal.into(), maximal.into());
            Self::try_new(minimal_ms, minimal_ms, maximal_ms)
        }
    }
}

impl TryFrom<(DurationHuman, DurationHuman, DurationHuman)> for DurationHumanValidator {
    type Error = DurationError;

    fn try_from(value: (DurationHuman, DurationHuman, DurationHuman)) -> Result<Self, Self::Error> {
        let (minimal, default, maximal) = &value;
        if minimal > default || maximal < default {
            Err(DurationError::DurationValidationMustBeOrdered {
                minimal: minimal.to_string(),
                default: default.to_string(),
                maximal: maximal.to_string(),
            })
        } else {
            let (minimal_ms, default_ms, maximal_ms): (u64, u64, u64) =
                (minimal.into(), default.into(), maximal.into());
            Self::try_new(minimal_ms, default_ms, maximal_ms)
        }
    }
}

impl TryFrom<(u64, u64, u64)> for DurationHumanValidator {
    type Error = DurationError;

    fn try_from(value: (u64, u64, u64)) -> Result<Self, Self::Error> {
        let min_def_max: (DurationHuman, DurationHuman, DurationHuman) = (
            (DurationHuman::from(value.0)),
            (DurationHuman::from(value.1)),
            (DurationHuman::from(value.2)),
        );
        Self::try_from(min_def_max)
    }
}

impl TryFrom<(&str, &str, &str)> for DurationHumanValidator {
    type Error = DurationError;

    fn try_from(value: (&str, &str, &str)) -> Result<Self, Self::Error> {
        let min_def_max: (DurationHuman, DurationHuman, DurationHuman) = (
            (DurationHuman::try_from(value.0)?),
            (DurationHuman::try_from(value.1)?),
            (DurationHuman::try_from(value.2)?),
        );

        Self::try_from(min_def_max)
    }
}

impl TryFrom<(u64, u64)> for DurationHumanValidator {
    type Error = DurationError;

    fn try_from(value: (u64, u64)) -> Result<Self, Self::Error> {
        let min_max: (DurationHuman, DurationHuman) = (
            (DurationHuman::from(value.0)),
            (DurationHuman::from(value.1)),
        );
        Self::try_from(min_max)
    }
}

impl TryFrom<(&str, &str)> for DurationHumanValidator {
    type Error = DurationError;

    fn try_from(value: (&str, &str)) -> Result<Self, Self::Error> {
        let min_max: (DurationHuman, DurationHuman) = (
            (DurationHuman::try_from(value.0)?),
            (DurationHuman::try_from(value.1)?),
        );

        Self::try_from(min_max)
    }
}

impl TryFrom<Vec<ParsedDuration>> for DurationHumanValidator {
    type Error = DurationError;

    fn try_from(value: Vec<ParsedDuration>) -> Result<Self, Self::Error> {
        let optionals: DurationRangeWithOptionalValues = value.into();
        optionals.min.map_or_else(
            || Err(DurationError::DurationValidationMinMustBeSpecified),
            |min| {
                optionals.max.map_or_else(
                    || Err(DurationError::DurationValidationMaxMustBeSpecified),
                    |max| {
                        optionals.default.map_or_else(
                            || Self::try_from((min, max)),
                            |default| Self::try_from((min, default, max)),
                        )
                    },
                )
            },
        )
    }
}
