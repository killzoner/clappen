use syn::spanned::Spanned;
use syn::{meta::ParseNestedMeta, LitStr, Result};

#[derive(Default)]
pub(crate) struct Attributes {
    pub prefix: String,
    pub default_prefix: String,
}

impl Attributes {
    pub fn parse(&mut self, meta: ParseNestedMeta) -> Result<()> {
        let Some(ident) = meta.path.get_ident() else {
            return Err(syn::Error::new(meta.path.span(), "expected an identifier"));
        };

        match ident.to_string().as_str() {
            "prefix" => {
                let prefix: LitStr = meta.value()?.parse()?;

                self.prefix = prefix.value();
            }
            "default_prefix" => {
                let prefix: LitStr = meta.value()?.parse()?;

                self.default_prefix = prefix.value();
            }
            _ => Err(syn::Error::new(ident.span(), "unknown attribute"))?,
        };

        Ok(())
    }
}
