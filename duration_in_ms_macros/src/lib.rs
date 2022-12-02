use duration_in_ms::{DurationInms, DurationInmsRangeAndDefault};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{braced, parse::Parse, parse_macro_input, Ident, Token};

#[proc_macro]
pub fn assign_duration_range(input: TokenStream) -> TokenStream {
    let params = parse_macro_input!(input as DurationRangeDeclaration);

    let range = &params.range;

    let (minimal, default, maximal) = (range.min, range.default, range.max);

    let err = if minimal > default || maximal < default {
        let (minimal, default, maximal): (String, String, String) = range.into();

        Some(format!(
            "Invalid range: should be {minimal} <= {default} <= {maximal}"
        ))
    } else if minimal < DurationInms::ONE_SECOND || default < DurationInms::ONE_SECOND {
        Some(format!(
            "duration {} must be at least {}",
            if minimal < DurationInms::ONE_SECOND {
                "min"
            } else {
                "default"
            },
            DurationInms::ONE_SECOND
        ))
    } else {
        None
    };

    let ident = format_ident!("{}", params.name.to_ascii_uppercase());

    TokenStream::from(err.map_or_else(|| {
            let (minimal_ms, default_ms, maximal_ms): (u64, u64, u64) = range.into();
                quote! {
                    const #ident: DurationInmsRangeAndDefault = DurationInmsRangeAndDefault::new(#minimal_ms, #default_ms, #maximal_ms);
                }
        }, |err| quote! {
            const #ident: DurationInmsRangeAndDefault = DurationInmsRangeAndDefault::new(1, 2, 3);
            compile_error!(#err);
        }))
}

struct DurationRangeDeclaration {
    name: String,
    range: DurationInmsRangeAndDefault,
}

impl Parse for DurationRangeDeclaration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let _punc: Token![=] = input.parse()?;
        let inner;
        braced!(inner in input);
        let range: DurationInmsRangeAndDefault = inner.parse()?;
        Ok(Self {
            name: name.to_string(),
            range,
        })
    }
}
