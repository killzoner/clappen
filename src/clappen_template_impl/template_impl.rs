use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemImpl;

use crate::clappen_template_impl::attrs;
use crate::clappen_template_impl::resolve::ResolvedTag;
use crate::clappen_template_impl::rewrite::substitute;
use crate::helper::{PrefixValue, prefix::StructPrefix};

// Entry point: resolve the two tags to concrete types, then rewrite the impl body. The attributes
// arrive checked. Addressing lives in `resolve`, the body rewrite in `rewrite`.
pub(crate) fn expand_template_impl(item: ItemImpl, attrs: attrs::Attributes) -> TokenStream {
    let attrs::Attributes {
        prefix,
        default_prefix,
        struct_ident,
        base_tag,
        prefixed_tag,
        chain,
        prefixed_fields,
    } = attrs;

    // debug doc: which instantiation this impl belongs to (prefix + nesting path, `[]` when not nested)
    let nesting: Vec<&str> = chain
        .iter()
        .filter_map(|step| step.command_prefix.value().as_deref())
        .collect();
    let doc = format!(
        " Template impl for `{struct_ident}` (prefix '{}', nested via [{}])",
        prefix.value().as_deref().unwrap_or_default(),
        nesting.join(".")
    );

    // an absent prefix means the base arm's child flatten call, where base is the struct's own
    // standalone type (drop the chain); a prefix means a prefixed instantiation, where base stays
    // nested (keep the chain).
    let base_chain: &[attrs::ChainStep] = match prefix.value() {
        Some(_) => &chain,
        None => &[],
    };
    let base = ResolvedTag::new(
        StructPrefix::default(),
        base_chain,
        &default_prefix,
        &struct_ident,
        base_tag,
    );
    // the prefixed instantiation starts from the prefix it was called with
    let prefixed = ResolvedTag::new(prefix, &chain, &default_prefix, &struct_ident, prefixed_tag);

    let expanded = substitute(item, base, prefixed, prefixed_fields);
    quote! {
        #[doc = #doc]
        #expanded
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;
    use quote::format_ident;
    use quote::quote;
    use syn::{Ident, parse_quote};

    use crate::clappen_template_impl::{DEFAULT_BASE_TAG, DEFAULT_PREFIXED_TAG};
    use crate::helper::{parse_literal, prefix::DefaultPrefix};

    fn ident(name: &str) -> Ident {
        Ident::new(name, Span::call_site())
    }

    fn struct_prefix(value: &str) -> StructPrefix {
        parse_literal(value)
    }

    // Attributes as the clappen macro forwards them: struct_ident set, one prefixed field `url`
    fn attributes(struct_ident: &str, prefix: StructPrefix) -> attrs::Attributes {
        attrs::Attributes {
            struct_ident: ident(struct_ident),
            prefix,
            prefixed_fields: vec![format_ident!("url")],
            default_prefix: DefaultPrefix::default(),
            base_tag: ident(DEFAULT_BASE_TAG),
            prefixed_tag: ident(DEFAULT_PREFIXED_TAG),
            chain: Vec::new(),
        }
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
        let out = expand_template_impl(item, attributes("ServerOptions", struct_prefix("svc")));

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
        let out = expand_template_impl(item, attributes("ServerOptions", struct_prefix("svc")));

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
            base_tag: ident("Src"),
            prefixed_tag: ident("Dst"),
            ..attributes("ServerOptions", struct_prefix("svc"))
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
            expand_template_impl(item, attrs).to_string(),
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
        // no prefix + a chain = the flatten-into-base conversion
        let attrs = attrs::Attributes {
            chain: vec![attrs::ChainStep {
                command_prefix: parse_literal("db"),
                field: ident("database"),
                parent_default: DefaultPrefix::default(),
            }],
            ..attributes("Db", StructPrefix::default())
        };

        let out = expand_template_impl(item, attrs);

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
