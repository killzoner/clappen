use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Attribute, ExprArray, Ident, LitStr, Meta, Result, Token, meta::ParseNestedMeta};

use crate::clappen_template_impl::{BASE_TAG_ATTR, PREFIXED_TAG_ATTR};

#[derive(Default)]
pub(crate) struct Attributes {
    pub prefix: String,
    // this struct's own default_prefix
    pub default_prefix: String,
    // canonical struct ident, forwarded automatically by the clappen macro
    pub struct_ident: Option<Ident>,
    // template tag idents, overridable by the user (default Base/Prefixed)
    pub base_tag: Option<Ident>,
    pub prefixed_tag: Option<Ident>,
    // nesting path from the top struct down to this one, one step per flatten level
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
            "default_prefix" => self.default_prefix = meta.value()?.parse::<LitStr>()?.value(),
            "struct_ident" => self.struct_ident = Some(meta.value()?.parse()?),
            BASE_TAG_ATTR => self.base_tag = Some(meta.value()?.parse()?),
            PREFIXED_TAG_ATTR => self.prefixed_tag = Some(meta.value()?.parse()?),
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

// the optional `base_tag = .., prefixed_tag = ..` overrides a user may put on a public
// `#[clappen_template_impl(..)]` marker; forwarded into the internal `Attributes`
#[derive(Default)]
pub(crate) struct TemplateTags {
    pub base_tag: Option<Ident>,
    pub prefixed_tag: Option<Ident>,
}

// read the overrides off the marker attribute; a bare `#[clappen_template_impl]` (no
// parentheses) is a `Meta::Path` with no args, so it yields the default (no overrides)
impl TryFrom<&Attribute> for TemplateTags {
    type Error = syn::Error;

    fn try_from(marker: &Attribute) -> syn::Result<Self> {
        match marker.meta {
            Meta::Path(_) => Ok(Self::default()),
            _ => marker.parse_args(),
        }
    }
}

impl Parse for TemplateTags {
    fn parse(input: ParseStream) -> Result<Self> {
        // comma-separated `key = value` pairs; parse_terminated handles the separators
        let pairs = Punctuated::<(Ident, Ident), Token![,]>::parse_terminated_with(input, |p| {
            let key: Ident = p.parse()?;
            p.parse::<Token![=]>()?;
            Ok((key, p.parse()?))
        })?;

        let mut tags = TemplateTags::default();
        for (key, value) in pairs {
            match key.to_string().as_str() {
                BASE_TAG_ATTR => tags.base_tag = Some(value),
                PREFIXED_TAG_ATTR => tags.prefixed_tag = Some(value),
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown attribute; expected `{BASE_TAG_ATTR}` or `{PREFIXED_TAG_ATTR}`"
                        ),
                    ));
                }
            }
        }
        Ok(tags)
    }
}

// emit the overrides as `base_tag = .., prefixed_tag = ..,` to forward into the template proc-macro
impl ToTokens for TemplateTags {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let base_key = format_ident!("{}", BASE_TAG_ATTR);
        let prefixed_key = format_ident!("{}", PREFIXED_TAG_ATTR);
        if let Some(t) = &self.base_tag {
            tokens.extend(quote! { #base_key = #t, });
        }
        if let Some(t) = &self.prefixed_tag {
            tokens.extend(quote! { #prefixed_key = #t, });
        }
    }
}
