use syn::spanned::Spanned;
use syn::{Result, meta::ParseNestedMeta};

use crate::helper;

#[derive(Default)]
pub(crate) struct Attributes {
    pub prefix: Option<String>,
    pub default_prefix: Option<String>,
}

impl Attributes {
    pub fn parse(&mut self, meta: ParseNestedMeta) -> Result<()> {
        let Some(ident) = meta.path.get_ident() else {
            return Err(syn::Error::new(meta.path.span(), "expected an identifier"));
        };

        match ident.to_string().as_str() {
            "prefix" => self.prefix = Some(helper::parse_prefix(&meta, ident)?),
            "default_prefix" => self.default_prefix = Some(helper::parse_prefix(&meta, ident)?),
            _ => Err(syn::Error::new(ident.span(), "unknown attribute"))?,
        };

        Ok(())
    }
}
