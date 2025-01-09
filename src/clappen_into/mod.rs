use quote::ToTokens;
use syn::meta::ParseNestedMeta;
use syn::spanned::Spanned;
use syn::ExprArray;

use crate::helper::parse_prefixes;

#[derive(Default, Clone)]
pub(crate) struct Attributes {
    pub fields: Vec<String>,
    pub prefixes: Vec<String>,
}

impl Attributes {
    pub fn parse(&mut self, meta: ParseNestedMeta) -> syn::Result<()> {
        let Some(ident) = meta.path.get_ident() else {
            return Err(syn::Error::new(meta.path.span(), "expected an identifier"));
        };

        match ident.to_string().as_str() {
            "fields" => {
                let attrs: ExprArray = meta.value()?.parse()?;

                self.fields = attrs
                    .elems
                    .iter()
                    .map(|e| e.into_token_stream().to_string())
                    .collect();
            }
            "prefixes" => {
                let arr: ExprArray = meta.value()?.parse()?;
                self.prefixes = parse_prefixes(arr)?
            }
            _ => Err(syn::Error::new(ident.span(), "unknown attribute"))?,
        };

        Ok(())
    }
}
