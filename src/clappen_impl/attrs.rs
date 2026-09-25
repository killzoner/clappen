use syn::spanned::Spanned;
use syn::{Ident, Result, meta::ParseNestedMeta};

use crate::helper::{
    self, DEFAULT_PREFIX_ATTR, PREFIX_ATTR,
    prefix::{DefaultPrefix, StructPrefix},
};

#[derive(Default)]
pub(crate) struct Attributes {
    pub prefix: StructPrefix,
    pub prefixed_fields: Vec<Ident>,
    pub default_prefix: DefaultPrefix,
}

impl Attributes {
    pub fn parse(&mut self, meta: ParseNestedMeta) -> Result<()> {
        let Some(ident) = meta.path.get_ident() else {
            return Err(syn::Error::new(meta.path.span(), "expected an identifier"));
        };

        match ident.to_string().as_str() {
            PREFIX_ATTR => self.prefix = meta.value()?.parse()?,
            "prefixed_fields" => self.prefixed_fields = helper::parse_bracketed(&meta)?,
            DEFAULT_PREFIX_ATTR => self.default_prefix = meta.value()?.parse()?,
            _ => Err(syn::Error::new(ident.span(), "unknown attribute"))?,
        };

        Ok(())
    }
}
