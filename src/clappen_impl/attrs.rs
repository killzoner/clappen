use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{meta::ParseNestedMeta, ExprArray, LitStr, Result};

#[derive(Default)]
pub(crate) struct Attributes {
    pub prefix: String,
    pub prefixed_fields: Vec<String>,
    pub default_prefix: String,
}

impl Attributes {
    pub fn parse(&mut self, meta: ParseNestedMeta) -> Result<()> {
        let Some(ident) = meta.path.get_ident() else {
            return Err(syn::Error::new(meta.path.span(), "expected an identifier"));
        };

        match ident.to_string().as_str() {
            "prefix" => {
                let prefix: LitStr = meta.value()?.parse()?; // don't use option type here, should be filled if specified

                self.prefix = prefix.value();
            }
            "prefixed_fields" => {
                let attrs: ExprArray = meta.value()?.parse()?;

                self.prefixed_fields = attrs
                    .elems
                    .iter()
                    .map(|e| e.into_token_stream().to_string())
                    .collect();
            }
            "default_prefix" => {
                let prefix: LitStr = meta.value()?.parse()?; // don't use option type here, should be filled if specified

                self.default_prefix = prefix.value();
            }
            _ => Err(syn::Error::new(ident.span(), "unknown attribute"))?,
        };

        Ok(())
    }
}
