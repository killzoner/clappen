use syn::spanned::Spanned;
use syn::LitStr;
use syn::{meta::ParseNestedMeta, Ident, Result};

#[derive(Default)]
pub(crate) struct Attributes {
    pub export: Option<Ident>,
    pub default_prefix: String,
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
            "default_prefix" => {
                let prefix: LitStr = meta.value()?.parse()?; // don't use option type here, should be filled if specified

                self.default_prefix = prefix.value();

                Ok(())
            }
            _ => Err(syn::Error::new(ident.span(), "unknown attribute")),
        }
    }
}
