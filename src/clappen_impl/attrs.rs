use syn::spanned::Spanned;
use syn::{Ident, Result, meta::ParseNestedMeta};

use crate::helper;

#[derive(Default)]
pub(crate) struct Attributes {
    pub prefix: Option<String>,
    pub prefixed_fields: Vec<Ident>,
    pub default_prefix: Option<String>,
}

impl Attributes {
    pub fn parse(&mut self, meta: ParseNestedMeta) -> Result<()> {
        let Some(ident) = meta.path.get_ident() else {
            return Err(syn::Error::new(meta.path.span(), "expected an identifier"));
        };

        match ident.to_string().as_str() {
            "prefix" => {
                self.prefix = Some(helper::require_non_empty(meta.value()?.parse()?, ident)?)
            }
            "prefixed_fields" => self.prefixed_fields = helper::parse_bracketed(&meta)?,
            "default_prefix" => {
                self.default_prefix =
                    Some(helper::require_non_empty(meta.value()?.parse()?, ident)?)
            }
            _ => Err(syn::Error::new(ident.span(), "unknown attribute"))?,
        };

        Ok(())
    }
}
