use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{ExprArray, Ident, LitStr, Result, Token, meta::ParseNestedMeta};

#[derive(Default)]
pub(crate) struct Attributes {
    pub prefix: String,
    // this struct's own default_prefix
    pub self_default: String,
    // canonical struct ident, forwarded automatically by the clappen macro
    pub item: Option<Ident>,
    // template tag idents, overridable by the user (default Base/Prefixed)
    pub base_tag: Option<Ident>,
    pub prefixed_tag: Option<Ident>,
    // nesting path from the outermost struct down to this one, one step per flatten level
    // (empty when flat)
    pub chain: Vec<ChainStep>,
    pub prefixed_fields: Vec<String>,
}

// one flatten level on the path: the field's command prefix, its name, and the parent struct's default_prefix
pub(crate) struct ChainStep {
    pub command_prefix: String,
    pub field: Ident,
    pub parent_default: String,
}

impl Attributes {
    pub fn parse(&mut self, meta: ParseNestedMeta) -> Result<()> {
        let Some(ident) = meta.path.get_ident() else {
            return Err(syn::Error::new(meta.path.span(), "expected an identifier"));
        };

        match ident.to_string().as_str() {
            "prefix" => self.prefix = meta.value()?.parse::<LitStr>()?.value(),
            "self_default" => self.self_default = meta.value()?.parse::<LitStr>()?.value(),
            "item" => self.item = Some(meta.value()?.parse()?),
            "base_tag" => self.base_tag = Some(meta.value()?.parse()?),
            "prefixed_tag" => self.prefixed_tag = Some(meta.value()?.parse()?),
            "chain" => {
                let value = meta.value()?;
                let content;
                syn::bracketed!(content in value);
                self.chain
                    .extend(Punctuated::<ChainStep, Token![,]>::parse_terminated(
                        &content,
                    )?);
            }
            "prefixed_fields" => {
                let attrs: ExprArray = meta.value()?.parse()?;

                self.prefixed_fields = attrs
                    .elems
                    .iter()
                    .map(|e| e.into_token_stream().to_string())
                    .collect();
            }
            _ => return Err(syn::Error::new(ident.span(), "unknown attribute")),
        };

        Ok(())
    }
}

// each element is a `(command_prefix, field, default)` tuple; the literal/ident fragments arrive
// wrapped in macro_rules' invisible groups, which the parse stream sees through
impl Parse for ChainStep {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        let command_prefix: LitStr = content.parse()?;
        content.parse::<Token![,]>()?;
        let field: Ident = content.parse()?;
        content.parse::<Token![,]>()?;
        let parent_default: LitStr = content.parse()?;
        Ok(ChainStep {
            command_prefix: command_prefix.value(),
            field,
            parent_default: parent_default.value(),
        })
    }
}
