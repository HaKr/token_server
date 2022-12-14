use crate::{DurationError, DurationHuman};

#[derive(Default)]
pub struct DurationHumanValidator {
    pub min: DurationHuman,
    pub default: DurationHuman,
    pub max: DurationHuman,
}

impl DurationHumanValidator {
    /// Create a new validator, with the given minimal, default and maximal durations
    ///
    /// ## Panics
    /// If any value < 1s, or if not: `minimal_nanos` <= `default_nanos` <= `maximal_nanos`
    #[must_use]
    pub const fn new(minimal_nanos: u64, default_nanos: u64, maximal_nanos: u64) -> Self {
        assert!(minimal_nanos <= default_nanos && default_nanos <= maximal_nanos);
        assert!(minimal_nanos >= DurationHuman::SEC);
        assert!(default_nanos >= DurationHuman::SEC);
        assert!(maximal_nanos >= DurationHuman::SEC);

        Self {
            min: DurationHuman::new(minimal_nanos),
            default: DurationHuman::new(default_nanos),
            max: DurationHuman::new(maximal_nanos),
        }
    }

    const fn try_new(
        minimal_nanos: u64,
        default_nanos: u64,
        maximal_nanos: u64,
    ) -> Result<Self, DurationError> {
        if minimal_nanos < DurationHuman::SEC {
            Err(DurationError::DurationValidationMinMustBeMoreThanOneSecond)
        } else if default_nanos < DurationHuman::SEC {
            Err(DurationError::DurationValidationDefaultMustBeMoreThanOneSecond)
        } else {
            Ok(Self {
                min: DurationHuman::new(minimal_nanos),
                default: DurationHuman::new(default_nanos),
                max: DurationHuman::new(maximal_nanos),
            })
        }
    }

    /// To be used as a `validate_parser` for clap
    ///
    /// ```compile_error
    ///  validate_parser = {|lifetime: &str|duration_range.parse_and_validate(lifetime)}
    /// ```
    /// # Errors
    ///
    /// Will return `Err` if duration is not within the given range
    /// permission to read it.
    pub fn parse_and_validate(&self, duration: &str) -> Result<DurationHuman, DurationError> {
        let duration_in_nanos = DurationHuman::try_from(duration)?;

        if self.contains(&duration_in_nanos) {
            Ok(duration_in_nanos)
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
