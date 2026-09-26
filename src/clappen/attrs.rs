use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream, Parser};
use syn::spanned::Spanned;
use syn::{Ident, Result};

use crate::helper::{DEFAULT_PREFIX_ATTR, prefix::DefaultPrefix};

pub(crate) struct Attributes {
    pub export: Ident,
    pub default_prefix: DefaultPrefix,
}

// `export` is mandatory, so holding an `Attributes` proves the macro name arrived
impl Parse for Attributes {
    fn parse(input: ParseStream) -> Result<Self> {
        // kept whole to span the error for a key that never arrives
        let args = input.parse::<TokenStream>()?;

        let mut export = None;
        let mut default_prefix = DefaultPrefix::default();

        syn::meta::parser(|meta| {
            let Some(ident) = meta.path.get_ident() else {
                return Err(syn::Error::new(meta.path.span(), "expected an identifier"));
            };

            match ident.to_string().as_str() {
                "export" => export = Some(meta.value()?.parse()?),
                DEFAULT_PREFIX_ATTR => default_prefix = meta.value()?.parse()?,
                _ => return Err(syn::Error::new(ident.span(), "unknown attribute")),
            };

            Ok(())
        })
        .parse2(args.clone())?;

        Ok(Self {
            export: export.ok_or_else(|| {
                syn::Error::new_spanned(&args, "clappen 'export' attribute not found")
            })?,
            default_prefix,
        })
    }
}
