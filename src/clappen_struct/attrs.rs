use syn::spanned::Spanned;
use syn::{Result, meta::ParseNestedMeta};

use crate::helper::{
    DEFAULT_PREFIX_ATTR, PREFIX_ATTR,
    prefix::{DefaultPrefix, StructPrefix},
};

#[derive(Default)]
pub(crate) struct Attributes {
    pub prefix: StructPrefix,
    pub default_prefix: DefaultPrefix,
}

impl Attributes {
    pub fn parse(&mut self, meta: ParseNestedMeta) -> Result<()> {
        let Some(ident) = meta.path.get_ident() else {
            return Err(syn::Error::new(meta.path.span(), "expected an identifier"));
        };

        match ident.to_string().as_str() {
            PREFIX_ATTR => self.prefix = meta.value()?.parse()?,
            DEFAULT_PREFIX_ATTR => self.default_prefix = meta.value()?.parse()?,
            _ => Err(syn::Error::new(ident.span(), "unknown attribute"))?,
        };

        Ok(())
    }
}
