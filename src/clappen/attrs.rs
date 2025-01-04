use syn::spanned::Spanned;
use syn::{LitBool, LitStr};
use syn::{meta::ParseNestedMeta, Ident, Result};

#[derive(Default)]
pub(crate) struct Attributes {
    pub export: Option<Ident>,
    pub default_prefix: String,
    pub gen_into: bool,
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
            "gen_into" => {
                let val: bool = meta
                                    .value()
                                    .and_then(|v| v.parse())
                                    .map(|v: LitBool| v.value())
                                    .unwrap_or(true); // if provided without value, true

                self.gen_into = val;
                Ok(())
            }
            _ => Err(syn::Error::new(ident.span(), "unknown attribute")),
        }
    }
}
