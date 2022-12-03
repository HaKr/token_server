use duration_in_ms::DurationInmsValidator;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{braced, parse::Parse, parse_macro_input, Ident, Token};

#[proc_macro]
pub fn assign_duration_range_validator(input: TokenStream) -> TokenStream {
    let assignment = parse_macro_input!(input as DurationRangeAssignment);

    let range = &assignment.range;

    let ident = format_ident!("{}", assignment.name.to_ascii_uppercase());
    let (minimal_ms, default_ms, maximal_ms): (u64, u64, u64) = range.into();

    TokenStream::from(quote! {
        const #ident: DurationInmsValidator = DurationInmsValidator::new(#minimal_ms, #default_ms, #maximal_ms);
    })
}

#[proc_macro]
pub fn duration_range_validator(input: TokenStream) -> TokenStream {
    let validator = parse_macro_input!(input as DurationInmsValidator);

    let (minimal_ms, default_ms, maximal_ms): (u64, u64, u64) = (&validator).into();

    TokenStream::from(quote! {
        DurationInmsValidator::new(#minimal_ms, #default_ms, #maximal_ms)
    })
}

#[proc_macro]
pub fn duration_range_value_parser(input: TokenStream) -> TokenStream {
    let validator = parse_macro_input!(input as DurationInmsValidator);

    let (minimal_ms, default_ms, maximal_ms): (u64, u64, u64) = (&validator).into();

    TokenStream::from(quote! {
        {|interval: &str|DurationInmsValidator::new(#minimal_ms, #default_ms, #maximal_ms).parse_and_validate(interval)}
    })
}

struct DurationRangeAssignment {
    name: String,
    range: DurationInmsValidator,
}

impl Parse for DurationRangeAssignment {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let _punc: Token![=] = input.parse()?;
        let inner;
        braced!(inner in input);
        let range: DurationInmsValidator = inner.parse()?;
        Ok(Self {
            name: name.to_string(),
            range,
        })
    }
}
