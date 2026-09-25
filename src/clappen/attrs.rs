use syn::spanned::Spanned;
use syn::{Ident, Result, meta::ParseNestedMeta};

use crate::helper::{DEFAULT_PREFIX_ATTR, prefix::DefaultPrefix};

#[derive(Default)]
pub(crate) struct Attributes {
    pub export: Option<Ident>,
    pub default_prefix: DefaultPrefix,
}

impl Attributes {
    pub fn parse(&mut self, meta: ParseNestedMeta) -> Result<()> {
        let Some(ident) = meta.path.get_ident() else {
            return Err(syn::Error::new(meta.path.span(), "expected an identifier"));
        };

        match ident.to_string().as_str() {
            "export" => {
                let op: Ident = meta.value()?.parse()?;
                self.export = Some(op);

                Ok(())
            }
            DEFAULT_PREFIX_ATTR => {
                self.default_prefix = meta.value()?.parse()?;

                Ok(())
            }
            _ => Err(syn::Error::new(ident.span(), "unknown attribute")),
        }
    }
}
