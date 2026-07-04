use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Attribute, Ident, LitStr, Meta, Result, Token, meta::ParseNestedMeta};

use crate::clappen_template_impl::{BASE_TAG_ATTR, PREFIXED_TAG_ATTR};
use crate::helper::{
    self, DEFAULT_PREFIX_ATTR, PREFIX_ATTR, PrefixValue,
    prefix::{CommandPrefix, DefaultPrefix, StructPrefix},
};

#[derive(Default)]
pub(crate) struct Attributes {
    pub prefix: StructPrefix,
    // this struct's own default_prefix
    pub default_prefix: DefaultPrefix,
    // canonical struct ident, forwarded automatically by the clappen macro
    pub struct_ident: Option<Ident>,
    // template tag idents, overridable by the user (default Base/Prefixed)
    pub base_tag: Option<Ident>,
    pub prefixed_tag: Option<Ident>,
    // nesting path from the top struct down to this one, one step per flatten level
    // (empty when flat)
    pub chain: Vec<ChainStep>,
    pub prefixed_fields: Vec<Ident>,
}

// one flatten level on the path: the field's command prefix, its name, and the parent struct's default_prefix
#[derive(Debug, PartialEq)]
pub(crate) struct ChainStep {
    pub command_prefix: CommandPrefix,
    pub field: Ident,
    pub parent_default: DefaultPrefix,
}

// one chain step as the macro writes it: the values a parent puts in a child's call, or the
// metavariables a chained arm forwards
pub(crate) enum ChainStepTokens<'a> {
    Concrete {
        command_prefix: &'a CommandPrefix,
        field: &'a Ident,
        parent_default: &'a DefaultPrefix,
    },
    Forwarded,
}

// `( command_prefix, field, parent_default )`, where each optional slot carries its own comma.
// The forwarded names are the ones the `@__template` matcher in `src/clappen/mod.rs` binds.
impl ToTokens for ChainStepTokens<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let (command_prefix, field, parent_default) = match self {
            Self::Concrete {
                command_prefix,
                field,
                parent_default,
            } => (
                command_prefix
                    .value()
                    .as_ref()
                    .map(|e| quote! { #e, })
                    .unwrap_or_default(),
                quote! { #field },
                parent_default
                    .value()
                    .as_ref()
                    .map(|e| quote! { , #e })
                    .unwrap_or_default(),
            ),
            Self::Forwarded => (
                quote! { $($command_prefix,)? },
                quote! { $field },
                quote! { $(, $parent_default)? },
            ),
        };
        tokens.extend(quote! { ( #command_prefix #field #parent_default ) });
    }
}

impl Attributes {
    pub fn parse(&mut self, meta: ParseNestedMeta) -> Result<()> {
        let Some(ident) = meta.path.get_ident() else {
            return Err(syn::Error::new(meta.path.span(), "expected an identifier"));
        };

        match ident.to_string().as_str() {
            PREFIX_ATTR => self.prefix = meta.value()?.parse()?,
            DEFAULT_PREFIX_ATTR => self.default_prefix = meta.value()?.parse()?,
            "struct_ident" => self.struct_ident = Some(meta.value()?.parse()?),
            BASE_TAG_ATTR => self.base_tag = Some(meta.value()?.parse()?),
            PREFIXED_TAG_ATTR => self.prefixed_tag = Some(meta.value()?.parse()?),
            "chain" => self
                .chain
                .extend(helper::parse_bracketed::<ChainStep>(&meta)?),
            "prefixed_fields" => self.prefixed_fields = helper::parse_bracketed(&meta)?,
            _ => return Err(syn::Error::new(ident.span(), "unknown attribute")),
        };

        Ok(())
    }
}

// each element is a `(command_prefix, field, default)` tuple whose two prefixes are optional, so a
// step can be `(field)`, `("p", field)`, `(field, "d")` or `("p", field, "d")`. The literal/ident
// fragments arrive wrapped in macro_rules' invisible groups, which the parse stream sees through.
impl Parse for ChainStep {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        syn::parenthesized!(content in input);

        // a leading literal is the field's command prefix; without one the step opens on the field
        let command_prefix = if content.peek(LitStr) {
            let command_prefix = content.parse()?;
            content.parse::<Token![,]>()?;
            command_prefix
        } else {
            CommandPrefix::default()
        };

        let field: Ident = content.parse()?;

        // the parent's default_prefix closes the step, and only when the parent has one
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

// the inverse of `Parse`: the concrete slot values, in the shape the parser reads back
impl ToTokens for ChainStep {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        ChainStepTokens::Concrete {
            command_prefix: &self.command_prefix,
            field: &self.field,
            parent_default: &self.parent_default,
        }
        .to_tokens(tokens);
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

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;

    fn step(command_prefix: Option<&str>, field: &str, parent_default: Option<&str>) -> ChainStep {
        ChainStep {
            command_prefix: command_prefix
                .map_or_else(CommandPrefix::default, helper::parse_literal),
            field: Ident::new(field, Span::call_site()),
            parent_default: parent_default
                .map_or_else(DefaultPrefix::default, helper::parse_literal),
        }
    }

    // every step shape the emitter writes must parse back to the step it came from
    #[test]
    fn chain_step_round_trips_through_its_tokens() {
        let cases = [
            step(None, "database", None),
            step(Some("p"), "database", None),
            step(None, "database", Some("d")),
            step(Some("p"), "database", Some("d")),
        ];

        for case in cases {
            let parsed: ChainStep = syn::parse2(case.to_token_stream()).unwrap();
            assert_eq!(parsed, case);
        }
    }
}
