use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, LitInt, Token,
};

use crate::{DurationError, DurationHuman, DurationHumanValidator};

struct ParsedDuration {
    arg: DurationRangeArgument,
    duration: DurationHuman,
}

enum DurationRangeArgument {
    Min,
    Default,
    Max,
}

impl Parse for DurationHuman {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let human_readable = input.parse::<LitInt>()?.to_string();

        TryInto::<Self>::try_into(human_readable.as_str())
            .map_err(|duration_error| input.error(duration_error.to_string()))
    }
}

impl Parse for DurationHumanValidator {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let parser = Punctuated::<ParsedDuration, Token![,]>::parse_separated_nonempty;
        let durations: Vec<ParsedDuration> = parser(input)?.into_iter().collect();

        TryInto::<Self>::try_into(durations).map_err(|duration_error| {
            println!("Validator failure: {}", duration_error);
            input.error(duration_error.to_string())
        })
    }
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
        let arg: DurationRangeArgument = input.parse()?;
        let _punc: Token![:] = input.parse()?;
        let duration: DurationHuman = input.parse()?;

        Ok(Self { arg, duration })
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
