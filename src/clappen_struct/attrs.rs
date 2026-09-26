use syn::Result;
use syn::parse::{Parse, ParseStream, Parser};
use syn::spanned::Spanned;

use crate::helper::{
    DEFAULT_PREFIX_ATTR, PREFIX_ATTR,
    prefix::{DefaultPrefix, StructPrefix},
};

pub(crate) struct Attributes {
    pub prefix: StructPrefix,
    pub default_prefix: DefaultPrefix,
}

impl Parse for Attributes {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut prefix = StructPrefix::default();
        let mut default_prefix = DefaultPrefix::default();

        syn::meta::parser(|meta| {
            let Some(ident) = meta.path.get_ident() else {
                return Err(syn::Error::new(meta.path.span(), "expected an identifier"));
            };

            match ident.to_string().as_str() {
                PREFIX_ATTR => prefix = meta.value()?.parse()?,
                DEFAULT_PREFIX_ATTR => default_prefix = meta.value()?.parse()?,
                _ => return Err(syn::Error::new(ident.span(), "unknown attribute")),
            };

            Ok(())
        })
        .parse2(input.parse()?)?;

        Ok(Self {
            prefix,
            default_prefix,
        })
    }
}
