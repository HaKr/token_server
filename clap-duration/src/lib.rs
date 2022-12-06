//! > Macros to add duration value parsers for clap arguments
//!
//!
//! ```rust
//! use clap::Parser;
//! use clap_duration::duration_range_value_parse;
//! use duration_human::{DurationHuman, DurationHumanValidator};
//!
//! #[derive(Parser)]
//! struct SampleOptions {
//!
//!     #[arg(
//!         long, default_value="666000ms",
//!         value_parser = duration_range_value_parse!(min: 10min, max: 1h)
//!     )]
//!     interval: DurationHuman,
//! }
//!
//! let opts = SampleOptions::parse();
//! assert_eq!(format!("{:#}",opts.interval), format!("11min 6s"));
//! assert_eq!(opts.interval.to_string(), "666s".to_string())
//!
// ```
use duration_human::DurationHumanValidator;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{braced, parse::Parse, parse_macro_input, Ident, Token};

/// macro
///
/// ## Example
/// ```rust
/// use clap::Parser;
/// use clap_duration::assign_duration_range_validator;
/// use duration_human::{DurationHuman, DurationHumanValidator};
///
/// assign_duration_range_validator!( LIFETIME_RANGE = {default: 333s, min: 1min, max: 60day});
///
/// #[derive(Parser)]
/// struct ServerOptions {
///    
///     #[arg(
///         long,
///         help = format!("What lifetime will it have, between {}", LIFETIME_RANGE),
///         default_value = LIFETIME_RANGE.default,
///         value_parser = {|lifetime: &str|LIFETIME_RANGE.parse_and_validate(lifetime)}
///     )]
///     lifetime: DurationHuman,
/// }
///
///  let opts = ServerOptions::parse();
///  assert_eq!(format!("{:#}",opts.lifetime), format!("5min 33s"));
///  assert_eq!(opts.lifetime.to_string(), "333s".to_string());
// ```
#[proc_macro]
pub fn assign_duration_range_validator(input: TokenStream) -> TokenStream {
    let assignment = parse_macro_input!(input as DurationRangeAssignment);

    let range = &assignment.range;

    let ident = format_ident!("{}", assignment.name.to_ascii_uppercase());
    let (minimal_ms, default_ms, maximal_ms): (u64, u64, u64) = range.into();

    TokenStream::from(quote! {
        const #ident: DurationHumanValidator = DurationHumanValidator::new(#minimal_ms, #default_ms, #maximal_ms);
    })
}

#[proc_macro]
pub fn duration_range_validator(input: TokenStream) -> TokenStream {
    let validator = parse_macro_input!(input as DurationHumanValidator);

    let (minimal_ms, default_ms, maximal_ms): (u64, u64, u64) = (&validator).into();

    TokenStream::from(quote! {
        DurationHumanValidator::new(#minimal_ms, #default_ms, #maximal_ms)
    })
}

/// macro for use as `value_parse` parameter in a clap arg attribute
///
/// ## Example
/// ```rust
/// # use clap::Parser;
/// # use clap_duration::duration_range_value_parse;
/// # use duration_human::{DurationHuman, DurationHumanValidator};
/// #
/// # #[derive(Parser)]
/// struct SampleOptions {
///     #[arg(
///         long, default_value="666000ms",
///         value_parser = duration_range_value_parse!(min: 10min, max: 1h)
///     )]
///     interval: DurationHuman,
/// }
///
/// let opts = SampleOptions::parse();
/// assert_eq!(format!("{:#}",opts.interval), format!("11min 6s"));
/// assert_eq!(opts.interval.to_string(), "666s".to_string())
///
// ```
#[proc_macro]
pub fn duration_range_value_parse(input: TokenStream) -> TokenStream {
    let validator = parse_macro_input!(input as DurationHumanValidator);

    let (minimal_ms, default_ms, maximal_ms): (u64, u64, u64) = (&validator).into();

    TokenStream::from(quote! {
        {|interval: &str|DurationHumanValidator::new(#minimal_ms, #default_ms, #maximal_ms).parse_and_validate(interval)}
    })
}

struct DurationRangeAssignment {
    name: String,
    range: DurationHumanValidator,
}

impl Parse for DurationRangeAssignment {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let _punc: Token![=] = input.parse()?;
        let inner;
        braced!(inner in input);
        let range: DurationHumanValidator = inner.parse()?;
        Ok(Self {
            name: name.to_string(),
            range,
        })
    }
}
