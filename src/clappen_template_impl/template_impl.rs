use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, ItemImpl};

use crate::clappen_template_impl::resolve::ResolvedTag;
use crate::clappen_template_impl::rewrite::substitute;
use crate::clappen_template_impl::{DEFAULT_BASE_TAG, DEFAULT_PREFIXED_TAG, attrs};

// Entry point: validate the forwarded attributes, resolve the two tags to concrete types, then
// rewrite the impl body. Addressing lives in `resolve`, the body rewrite in `rewrite`.
pub(crate) fn expand_template_impl(
    item: ItemImpl,
    attrs: attrs::Attributes,
) -> syn::Result<TokenStream> {
    let Some(struct_ident) = attrs.struct_ident else {
        return Err(syn::Error::new(
            Span::call_site(),
            "clappen_template_impl requires `struct_ident`",
        ));
    };

    let base_tag = attrs
        .base_tag
        .unwrap_or_else(|| Ident::new(DEFAULT_BASE_TAG, Span::call_site()));
    let prefixed_tag = attrs
        .prefixed_tag
        .unwrap_or_else(|| Ident::new(DEFAULT_PREFIXED_TAG, Span::call_site()));

    // the prefixed instantiation uses the literal prefix, the base the empty one
    let prefixed = ResolvedTag::new(
        &attrs.prefix,
        &attrs.chain,
        &attrs.default_prefix,
        &struct_ident,
        prefixed_tag,
    );
    // prefix is empty only in the base arm's child flatten call, where base is the struct's
    // own standalone type (drop the chain); a non-empty prefix is a prefixed instantiation,
    // where base stays nested (keep the chain).
    let base_is_standalone = attrs.prefix.is_empty();
    let base_chain: &[attrs::ChainStep] = if base_is_standalone {
        &[]
    } else {
        &attrs.chain
    };
    let base = ResolvedTag::new(
        "",
        base_chain,
        &attrs.default_prefix,
        &struct_ident,
        base_tag,
    );

    // debug doc: which instantiation this impl belongs to (prefix + nesting path, `[]` when not nested)
    let nesting: Vec<&str> = attrs
        .chain
        .iter()
        .map(|step| step.command_prefix.as_str())
        .filter(|command| !command.is_empty())
        .collect();
    let doc = format!(
        " Template impl for `{struct_ident}` (prefix '{}', nested via [{}])",
        attrs.prefix,
        nesting.join(".")
    );

    let expanded = substitute(item, base, prefixed, attrs.prefixed_fields);
    Ok(quote! {
        #[doc = #doc]
        #expanded
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::parse_quote;

    fn ident(name: &str) -> Ident {
        Ident::new(name, Span::call_site())
    }

    // Attributes as the clappen macro forwards them: struct_ident set, one prefixed field `url`
    fn attributes(struct_ident: &str, prefix: &str) -> attrs::Attributes {
        attrs::Attributes {
            struct_ident: Some(ident(struct_ident)),
            prefix: prefix.to_string(),
            prefixed_fields: vec!["url".to_string()],
            ..Default::default()
        }
    }

    #[test]
    fn expand_template_impl_requires_struct_ident() {
        let item: ItemImpl = parse_quote! {
            impl From<Prefixed> for Base {
                fn from(value: Prefixed) -> Self { Self { url: value.url } }
            }
        };
        let err = expand_template_impl(item, attrs::Attributes::default()).unwrap_err();
        assert_eq!(
            err.to_string(),
            "clappen_template_impl requires `struct_ident`"
        );
    }

    #[test]
    fn expand_template_impl_scopes_bindings_per_method() {
        // two methods reuse the param name `value` with different tags; each method resolves its
        // own binding, so `a` prefixes the field (Prefixed) and `b` leaves it bare (Base)
        let item: ItemImpl = parse_quote! {
            impl Base {
                fn a(value: Prefixed) -> Self { Self { url: value.url } }
                fn b(value: Base) -> Self { Self { url: value.url } }
            }
        };
        let out = expand_template_impl(item, attributes("ServerOptions", "svc")).unwrap();

        let expected = quote! {
            #[doc = " Template impl for `ServerOptions` (prefix 'svc', nested via [])"]
            impl ServerOptions {
                fn a(value: SvcServerOptions) -> Self { Self { url: value.svc_url } }
                fn b(value: ServerOptions) -> Self { Self { url: value.url } }
            }
        };
        assert_eq!(out.to_string(), expected.to_string());
    }

    #[test]
    fn expand_template_impl_rewrites_tags_and_prefixes_fields() {
        let item: ItemImpl = parse_quote! {
            impl From<Prefixed> for Base {
                fn from(value: Prefixed) -> Self { Self { url: value.url } }
            }
        };
        let out = expand_template_impl(item, attributes("ServerOptions", "svc")).unwrap();

        // Base -> the bare struct, Prefixed -> the prefixed struct, the `Prefixed` binding's
        // field read is prefixed, and the `Self` (Base) literal keeps the unprefixed field
        let expected = quote! {
            #[doc = " Template impl for `ServerOptions` (prefix 'svc', nested via [])"]
            impl From<SvcServerOptions> for ServerOptions {
                fn from(value: SvcServerOptions) -> Self {
                    Self { url: value.svc_url }
                }
            }
        };
        assert_eq!(out.to_string(), expected.to_string());
    }

    #[test]
    fn expand_template_impl_honors_custom_tags() {
        let item: ItemImpl = parse_quote! {
            impl From<Dst> for Src {
                fn from(value: Dst) -> Self { Self { url: value.url } }
            }
        };
        let attrs = attrs::Attributes {
            base_tag: Some(ident("Src")),
            prefixed_tag: Some(ident("Dst")),
            ..attributes("ServerOptions", "svc")
        };

        // custom tag idents resolve to the same output as the default `Base`/`Prefixed`
        let expected = quote! {
            #[doc = " Template impl for `ServerOptions` (prefix 'svc', nested via [])"]
            impl From<SvcServerOptions> for ServerOptions {
                fn from(value: SvcServerOptions) -> Self {
                    Self { url: value.svc_url }
                }
            }
        };
        assert_eq!(
            expand_template_impl(item, attrs).unwrap().to_string(),
            expected.to_string()
        );
    }

    #[test]
    fn expand_template_impl_flatten_into_base_ignores_chain_for_the_base() {
        let item: ItemImpl = parse_quote! {
            impl From<Prefixed> for Base {
                fn from(value: Prefixed) -> Self { Self { url: value.url } }
            }
        };
        // empty prefix + a chain = the flatten-into-base conversion
        let attrs = attrs::Attributes {
            chain: vec![attrs::ChainStep {
                command_prefix: "db".to_string(),
                field: ident("database"),
                parent_default: String::new(),
            }],
            ..attributes("Db", "")
        };

        let out = expand_template_impl(item, attrs).unwrap();

        // base (Self) is the struct's own standalone type (chain ignored); prefixed walks the
        // chain to the nested type and prefixes the field read
        let expected = quote! {
            #[doc = " Template impl for `Db` (prefix '', nested via [db])"]
            impl From<__inner_database::DbDb> for Db {
                fn from(value: __inner_database::DbDb) -> Self {
                    Self { url: value.db_url }
                }
            }
        };
        assert_eq!(out.to_string(), expected.to_string());
    }
}
