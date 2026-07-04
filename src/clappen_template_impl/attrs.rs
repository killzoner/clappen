use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::parse::{Parse, ParseStream, Parser};
use syn::spanned::Spanned;
use syn::{Attribute, Ident, LitStr, Meta, Result, Token};

use crate::clappen_template_impl::{
    BASE_TAG_ATTR, CHAIN_ATTR, DEFAULT_BASE_TAG, DEFAULT_PREFIXED_TAG, PREFIXED_TAG_ATTR,
    STRUCT_IDENT_ATTR,
};
use crate::helper::{
    self, DEFAULT_PREFIX_ATTR, PREFIX_ATTR, PREFIXED_FIELDS_ATTR,
    prefix::{CommandPrefix, DefaultPrefix, StructPrefix},
};

pub(crate) struct Attributes {
    pub prefix: StructPrefix,
    // this struct's own default_prefix
    pub default_prefix: DefaultPrefix,
    // the struct ident, forwarded by the clappen macro
    pub struct_ident: Ident,
    // the tag idents: the user's overrides, or `Base`/`Prefixed`
    pub base_tag: Ident,
    pub prefixed_tag: Ident,
    // one step per flatten level, from the top struct down to this one (empty when flat)
    pub chain: Vec<ChainStep>,
    pub prefixed_fields: Vec<Ident>,
}

// one flatten level: the field's command prefix, its name, and the parent's default_prefix
pub(crate) struct ChainStep {
    pub command_prefix: CommandPrefix,
    pub field: Ident,
    pub parent_default: DefaultPrefix,
}

impl Parse for Attributes {
    fn parse(input: ParseStream) -> Result<Self> {
        // kept for the error span when `struct_ident` is missing
        let args = input.parse::<TokenStream>()?;

        let mut prefix = StructPrefix::default();
        let mut default_prefix = DefaultPrefix::default();
        let mut struct_ident = None;
        let mut base_tag = None;
        let mut prefixed_tag = None;
        let mut chain: Vec<ChainStep> = Vec::new();
        let mut prefixed_fields = Vec::new();

        syn::meta::parser(|meta| {
            let Some(ident) = meta.path.get_ident() else {
                return Err(syn::Error::new(meta.path.span(), "expected an identifier"));
            };

            match ident.to_string().as_str() {
                PREFIX_ATTR => prefix = meta.value()?.parse()?,
                DEFAULT_PREFIX_ATTR => default_prefix = meta.value()?.parse()?,
                STRUCT_IDENT_ATTR => struct_ident = Some(meta.value()?.parse()?),
                BASE_TAG_ATTR => base_tag = Some(meta.value()?.parse()?),
                PREFIXED_TAG_ATTR => prefixed_tag = Some(meta.value()?.parse()?),
                CHAIN_ATTR => chain.extend(helper::parse_bracketed::<ChainStep>(&meta)?),
                PREFIXED_FIELDS_ATTR => prefixed_fields = helper::parse_bracketed(&meta)?,
                _ => return Err(syn::Error::new(ident.span(), "unknown attribute")),
            };

            Ok(())
        })
        .parse2(args.clone())?;

        let struct_ident = struct_ident.ok_or_else(|| {
            syn::Error::new_spanned(
                &args,
                format!("clappen '{STRUCT_IDENT_ATTR}' attribute not found"),
            )
        })?;

        Ok(Self {
            prefix,
            default_prefix,
            struct_ident,
            base_tag: base_tag.unwrap_or_else(|| Ident::new(DEFAULT_BASE_TAG, Span::call_site())),
            prefixed_tag: prefixed_tag
                .unwrap_or_else(|| Ident::new(DEFAULT_PREFIXED_TAG, Span::call_site())),
            chain,
            prefixed_fields,
        })
    }
}

// `(field)`, `("p", field)`, `(field, "d")` or `("p", field, "d")`. The fragments arrive in
// macro_rules' invisible groups, which the parse stream sees through.
impl Parse for ChainStep {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        syn::parenthesized!(content in input);

        // an optional leading literal: the field's command prefix
        let command_prefix = if content.peek(LitStr) {
            let command_prefix = content.parse()?;
            content.parse::<Token![,]>()?;
            command_prefix
        } else {
            CommandPrefix::default()
        };

        let field: Ident = content.parse()?;

        // an optional closing literal: the parent's default_prefix
        let parent_default = if content.is_empty() {
            DefaultPrefix::default()
        } else {
            content.parse::<Token![,]>()?;
            content.parse()?
        };

        Ok(ChainStep {
            command_prefix,
            field,
            parent_default,
        })
    }
}

// the user's `base_tag = .., prefixed_tag = ..` overrides on the marker
#[derive(Default)]
pub(crate) struct TemplateTags {
    pub base_tag: Option<Ident>,
    pub prefixed_tag: Option<Ident>,
}

// a bare `#[clappen_template_impl]` is a `Meta::Path`: no overrides
impl TryFrom<&Attribute> for TemplateTags {
    type Error = syn::Error;

    fn try_from(marker: &Attribute) -> syn::Result<Self> {
        let mut tags = Self::default();
        if let Meta::Path(_) = marker.meta {
            return Ok(tags);
        }

        marker.parse_nested_meta(|meta| {
            let Some(ident) = meta.path.get_ident() else {
                return Err(syn::Error::new(meta.path.span(), "expected an identifier"));
            };

            match ident.to_string().as_str() {
                BASE_TAG_ATTR => tags.base_tag = Some(meta.value()?.parse()?),
                PREFIXED_TAG_ATTR => tags.prefixed_tag = Some(meta.value()?.parse()?),
                _ => return Err(syn::Error::new(ident.span(), "unknown attribute")),
            };

            Ok(())
        })?;

        Ok(tags)
    }
}

// the overrides as `base_tag = .., prefixed_tag = ..,`, forwarded to the proc-macro
impl ToTokens for TemplateTags {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let base_key = format_ident!("{}", BASE_TAG_ATTR);
        let prefixed_key = format_ident!("{}", PREFIXED_TAG_ATTR);
        let base_tag = self.base_tag.iter();
        let prefixed_tag = self.prefixed_tag.iter();
        tokens.extend(quote! { #(#base_key = #base_tag,)* #(#prefixed_key = #prefixed_tag,)* });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // `struct_ident` has no default, so the attributes cannot be built without it
    #[test]
    fn attributes_require_struct_ident() {
        let Err(err) = syn::parse2::<Attributes>(quote! { prefix = "svc" }) else {
            panic!("`struct_ident` is mandatory, so parsing without it must fail");
        };
        assert_eq!(
            err.to_string(),
            "clappen 'struct_ident' attribute not found"
        );
    }
}
