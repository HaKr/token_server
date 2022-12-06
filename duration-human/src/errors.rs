use std::num::ParseIntError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DurationError {
    #[error("Duration must lie between {range}")]
    DurationMustLieBetween { range: String },

    #[error("Duration must be specified as a positive integer, immediately followed by days, h, min, s, ms, μs or ns")]
    InvalidSyntax,

    #[error("Invalid duration value")]
    InvalidValue {
        #[from]
        source: ParseIntError,
    },

    #[error("'{sym}' is not supported as a duration symbol")]
    UnsupportedSymbol { sym: String },

    #[error("Invalid range: should be {minimal} <=  {maximal}")]
    DurationValidationMinMustBeLessOrEqualMax { minimal: String, maximal: String },

    #[error("Invalid range: should be {minimal} <= {default} <= {maximal}")]
    DurationValidationMustBeOrdered {
        minimal: String,
        default: String,
        maximal: String,
    },

    #[error("could not find min duration")]
    DurationValidationMinMustBeSpecified,

    #[error("could not find max duration")]
    DurationValidationMaxMustBeSpecified,

    #[error("min duration must be 1s or longer")]
    DurationValidationMinMustBeMoreThanOneSecond,

    #[error("default duration must be 1s or longer")]
    DurationValidationDefaultMustBeMoreThanOneSecond,
}
