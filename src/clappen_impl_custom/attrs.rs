use syn::spanned::Spanned;
use syn::{LitStr, Result, meta::ParseNestedMeta};

#[derive(Default)]
pub(crate) struct Attributes {
    pub ignore_self: Option<String>,
}

impl Attributes {
    pub fn parse(&mut self, meta: ParseNestedMeta) -> Result<()> {
        let Some(ident) = meta.path.get_ident() else {
            return Err(syn::Error::new(meta.path.span(), "expected an identifier"));
        };

        match ident.to_string().as_str() {
            "ignore_self" => {
                let ignore_self: Option<LitStr> = meta.value()?.parse()?; // don't use option type here, should be filled if specified

                self.ignore_self = ignore_self.map(|e| e.value());
            }
            _ => Err(syn::Error::new(ident.span(), "unknown attribute"))?,
        };

        Ok(())
    }
}
