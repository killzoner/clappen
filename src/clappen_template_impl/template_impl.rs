use proc_macro2::TokenStream;
use quote::quote;
use syn::visit_mut::VisitMut;
use syn::{Ident, ItemImpl};

use crate::clappen_template_impl::{
    attrs,
    resolve::ResolvedTag,
    rewrite::{PrefixFields, ReplaceTags, Tags},
};
use crate::helper::{PrefixValue, prefix::StructPrefix};

// Resolves the two tags to their types, then rewrites the impl body.
pub(crate) fn expand(item: ItemImpl, attrs: attrs::Attributes) -> syn::Result<TokenStream> {
    let attrs::Attributes {
        prefix,
        default_prefix,
        struct_ident,
        base_tag,
        prefixed_tag,
        chain,
        prefixed_fields,
    } = attrs;

    // debug doc: the prefix and the nesting path of this instantiation
    let nesting: Vec<&str> = chain
        .iter()
        .filter_map(|step| step.command_prefix.value().as_deref())
        .collect();
    let doc = format!(
        " Template impl for `{struct_ident}` (prefix '{}', nested via [{}])",
        prefix.as_str(),
        nesting.join(".")
    );

    // no prefix: the base arm's call into a child, where base is the standalone type
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
    let prefixed = ResolvedTag::new(prefix, &chain, &default_prefix, &struct_ident, prefixed_tag);

    let expanded = substitute(item, base, prefixed, prefixed_fields);
    Ok(quote! {
        #[doc = #doc]
        #expanded
    })
}

// prefixes the fields of tag-typed values, then replaces the tags with their paths
fn substitute(
    mut item: ItemImpl,
    base: ResolvedTag,
    prefixed: ResolvedTag,
    fields: Vec<Ident>,
) -> ItemImpl {
    let tags = Tags { base, prefixed };

    let mut prefix_fields = PrefixFields::new(&tags, fields, &item);
    prefix_fields.visit_item_impl_mut(&mut item);
    ReplaceTags { tags: &tags }.visit_item_impl_mut(&mut item);

    item
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::format_ident;
    use syn::parse_quote;

    use crate::clappen_template_impl::{DEFAULT_BASE_TAG, DEFAULT_PREFIXED_TAG};
    use crate::helper::{parse_literal, prefix::DefaultPrefix};

    // Attributes as the clappen macro forwards them: struct_ident set, one prefixed field `url`
    fn attributes(struct_ident: &str, prefix: StructPrefix) -> attrs::Attributes {
        attrs::Attributes {
            struct_ident: format_ident!("{struct_ident}"),
            prefix,
            prefixed_fields: vec![format_ident!("url")],
            default_prefix: DefaultPrefix::default(),
            base_tag: format_ident!("{DEFAULT_BASE_TAG}"),
            prefixed_tag: format_ident!("{DEFAULT_PREFIXED_TAG}"),
            chain: Vec::new(),
        }
    }

    // the whole rewrite, with `Base` as `ServerOptions`, `Prefixed` as `SvcServerOptions`, and
    // `url` as the only prefixed field
    fn assert_rewrites(item: ItemImpl, expected: ItemImpl) {
        let resolve = |prefix, name| {
            ResolvedTag::new(
                prefix,
                &[],
                &DefaultPrefix::default(),
                &format_ident!("ServerOptions"),
                format_ident!("{name}"),
            )
        };
        let base = resolve(StructPrefix::default(), "Base");
        let prefixed = resolve(parse_literal("svc"), "Prefixed");
        let rewritten = substitute(item, base, prefixed, vec![format_ident!("url")]);

        assert_eq!(
            quote!(#rewritten).to_string(),
            quote!(#expected).to_string()
        );
    }

    #[test]
    fn expand_scopes_bindings_per_method() {
        // `value` is `Prefixed` in `a` and `Base` in `b`, so only `a` prefixes the field
        let item: ItemImpl = parse_quote! {
            impl Base {
                fn a(value: Prefixed) -> Self { Self { url: value.url } }
                fn b(value: Base) -> Self { Self { url: value.url } }
            }
        };
        let out = expand(item, attributes("ServerOptions", parse_literal("svc"))).unwrap();

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
    fn expand_rewrites_tags_and_prefixes_fields() {
        let item: ItemImpl = parse_quote! {
            impl From<Prefixed> for Base {
                fn from(value: Prefixed) -> Self { Self { url: value.url } }
            }
        };
        let out = expand(item, attributes("ServerOptions", parse_literal("svc"))).unwrap();

        // the `Self` literal is the base, so its field stays unprefixed
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
    fn expand_honors_custom_tags() {
        let item: ItemImpl = parse_quote! {
            impl From<Dst> for Src {
                fn from(value: Dst) -> Self { Self { url: value.url } }
            }
        };
        let attrs = attrs::Attributes {
            base_tag: format_ident!("Src"),
            prefixed_tag: format_ident!("Dst"),
            ..attributes("ServerOptions", parse_literal("svc"))
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
            expand(item, attrs).unwrap().to_string(),
            expected.to_string()
        );
    }

    #[test]
    fn expand_flatten_into_base_ignores_chain_for_the_base() {
        let item: ItemImpl = parse_quote! {
            impl From<Prefixed> for Base {
                fn from(value: Prefixed) -> Self { Self { url: value.url } }
            }
        };
        // no prefix + a chain = the flatten-into-base conversion
        let attrs = attrs::Attributes {
            chain: vec![attrs::ChainStep {
                command_prefix: parse_literal("db"),
                field: format_ident!("nested"),
                parent_default: DefaultPrefix::default(),
            }],
            ..attributes("MyStruct", StructPrefix::default())
        };

        let out = expand(item, attrs).unwrap();

        // base is the standalone type, prefixed walks the chain
        let expected = quote! {
            #[doc = " Template impl for `MyStruct` (prefix '', nested via [db])"]
            impl From<__inner_nested::DbMyStruct> for MyStruct {
                fn from(value: __inner_nested::DbMyStruct) -> Self {
                    Self { url: value.db_url }
                }
            }
        };
        assert_eq!(out.to_string(), expected.to_string());
    }

    #[test]
    fn a_param_binds_only_inside_its_own_method() {
        // `b` reads a `value` it does not bind, so `a`'s param must not reach it
        assert_rewrites(
            parse_quote! {
                impl Base {
                    fn a(value: Prefixed) -> String { value.url }
                    fn b() -> String { value.url }
                }
            },
            parse_quote! {
                impl ServerOptions {
                    fn a(value: SvcServerOptions) -> String { value.svc_url }
                    fn b() -> String { value.url }
                }
            },
        );
    }

    #[test]
    fn a_shadowing_let_stops_the_prefixing_of_that_name() {
        assert_rewrites(
            parse_quote! {
                impl From<Prefixed> for Base {
                    fn from(value: Prefixed) -> Self {
                        // `value` is a `Fallback` from here on, so its field keeps its own name
                        let value = Fallback::default();
                        Self { url: value.url }
                    }
                }
            },
            parse_quote! {
                impl From<SvcServerOptions> for ServerOptions {
                    fn from(value: SvcServerOptions) -> Self {
                        let value = Fallback::default();
                        Self { url: value.url }
                    }
                }
            },
        );
    }

    #[test]
    fn a_let_shadows_up_to_the_end_of_its_own_block_only() {
        assert_rewrites(
            parse_quote! {
                impl From<Prefixed> for Base {
                    fn from(value: Prefixed) -> Self {
                        let inner = {
                            let value = Fallback::default();
                            value.url
                        };
                        Self { url: value.url }
                    }
                }
            },
            parse_quote! {
                impl From<SvcServerOptions> for ServerOptions {
                    fn from(value: SvcServerOptions) -> Self {
                        let inner = {
                            let value = Fallback::default();
                            value.url
                        };
                        Self { url: value.svc_url }
                    }
                }
            },
        );
    }

    #[test]
    fn a_let_of_a_tag_type_binds_that_tag() {
        assert_rewrites(
            parse_quote! {
                impl From<Base> for Prefixed {
                    fn from(value: Base) -> Self {
                        let value: Prefixed = build();
                        Prefixed { url: value.url }
                    }
                }
            },
            parse_quote! {
                impl From<ServerOptions> for SvcServerOptions {
                    fn from(value: ServerOptions) -> Self {
                        let value: SvcServerOptions = build();
                        SvcServerOptions { svc_url: value.svc_url }
                    }
                }
            },
        );
    }

    #[test]
    fn a_closure_param_shadows_or_binds_like_a_let() {
        assert_rewrites(
            parse_quote! {
                impl From<Prefixed> for Base {
                    fn from(value: Prefixed) -> Self {
                        let plain = |value: Fallback| value.url;
                        let tagged = |value: Prefixed| value.url;
                        Self { url: value.url }
                    }
                }
            },
            parse_quote! {
                impl From<SvcServerOptions> for ServerOptions {
                    fn from(value: SvcServerOptions) -> Self {
                        let plain = |value: Fallback| value.url;
                        let tagged = |value: SvcServerOptions| value.svc_url;
                        Self { url: value.svc_url }
                    }
                }
            },
        );
    }

    #[test]
    fn a_for_pattern_and_a_match_arm_shadow_for_their_own_body_only() {
        assert_rewrites(
            parse_quote! {
                impl From<Prefixed> for Base {
                    fn from(value: Prefixed) -> Self {
                        for value in value.url {
                            drop(value.url);
                        }
                        let url = match other {
                            Some(value) => value.url,
                            None => value.url,
                        };
                        Self { url }
                    }
                }
            },
            parse_quote! {
                impl From<SvcServerOptions> for ServerOptions {
                    fn from(value: SvcServerOptions) -> Self {
                        for value in value.svc_url {
                            drop(value.url);
                        }
                        let url = match other {
                            Some(value) => value.url,
                            None => value.svc_url,
                        };
                        Self { url }
                    }
                }
            },
        );
    }
}
