use proc_macro2::TokenStream;
use quote::quote;
use syn::visit_mut::VisitMut;
use syn::{Ident, ItemImpl};

use crate::clappen_template_impl::{
    attrs,
    resolve::{ResolvedTag, Tags},
    rewrite::{PrefixFields, ReplaceTags},
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

    // `Base` is always the root type, at any depth
    let base = ResolvedTag::new(
        StructPrefix::default(),
        &[],
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

    let mut prefix_fields = PrefixFields::new(&tags, fields);
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
    fn expand_nested_with_a_prefix_keeps_the_root_as_base() {
        let item: ItemImpl = parse_quote! {
            impl From<Prefixed> for Base {
                fn from(value: Prefixed) -> Self { Self { url: value.url } }
            }
        };
        let attrs = attrs::Attributes {
            chain: vec![attrs::ChainStep {
                command_prefix: parse_literal("db"),
                field: format_ident!("nested"),
                parent_default: DefaultPrefix::default(),
            }],
            ..attributes("MyStruct", parse_literal("test1"))
        };

        let out = expand(item, attrs).unwrap();

        // base is the root, not the parent's field type, prefixed walks the chain
        let expected = quote! {
            #[doc = " Template impl for `MyStruct` (prefix 'test1', nested via [db])"]
            impl From<__inner_test1_nested::Test1DbMyStruct> for MyStruct {
                fn from(value: __inner_test1_nested::Test1DbMyStruct) -> Self {
                    Self { url: value.test1_db_url }
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
    fn a_typed_closure_param_binds_like_a_typed_let() {
        assert_rewrites(
            parse_quote! {
                impl From<Prefixed> for Base {
                    fn from(value: Prefixed) -> Self {
                        let urls: Vec<String> = others.iter().map(|other: &Prefixed| other.url.clone()).collect();
                        Self { url: value.url }
                    }
                }
            },
            parse_quote! {
                impl From<SvcServerOptions> for ServerOptions {
                    fn from(value: SvcServerOptions) -> Self {
                        let urls: Vec<String> = others.iter().map(|other: &SvcServerOptions| other.svc_url.clone()).collect();
                        Self { url: value.svc_url }
                    }
                }
            },
        );
    }

    #[test]
    fn macro_args_are_rewritten_like_the_code_around_them() {
        assert_rewrites(
            parse_quote! {
                impl From<Prefixed> for Base {
                    fn from(value: Prefixed) -> Self {
                        let all: Vec<Base> = vec![Base::default()];
                        assert_eq!(value.url, String::new());
                        Self { url: value.url }
                    }
                }
            },
            parse_quote! {
                impl From<SvcServerOptions> for ServerOptions {
                    fn from(value: SvcServerOptions) -> Self {
                        let all: Vec<ServerOptions> = vec![ServerOptions::default()];
                        assert_eq!(value.svc_url, String::new());
                        Self { url: value.svc_url }
                    }
                }
            },
        );
    }

    #[test]
    fn a_macro_body_that_syn_cannot_parse_is_rewritten_token_by_token() {
        assert_rewrites(
            parse_quote! {
                impl From<Prefixed> for Base {
                    fn from(value: Prefixed) -> Self {
                        let renamed = matches!(other, Some(x) if x == value.url && x == Base::X);
                        let kept = matches!(other, Some(x) if x == value.url() && x == other.value.url && x == other::Base::X);
                        Self { url: value.url }
                    }
                }
            },
            parse_quote! {
                impl From<SvcServerOptions> for ServerOptions {
                    fn from(value: SvcServerOptions) -> Self {
                        let renamed = matches!(other, Some(x) if x == value.svc_url && x == ServerOptions::X);
                        let kept = matches!(other, Some(x) if x == value.url() && x == other.value.url && x == other::Base::X);
                        Self { url: value.svc_url }
                    }
                }
            },
        );
    }

    #[test]
    fn a_macro_body_of_statements_is_rewritten_like_a_block() {
        assert_rewrites(
            parse_quote! {
                impl From<Prefixed> for Base {
                    fn from(value: Prefixed) -> Self {
                        let all: Vec<Prefixed> = vec![Prefixed { url: value.url.clone() }; 2];
                        Self { url: value.url }
                    }
                }
            },
            parse_quote! {
                impl From<SvcServerOptions> for ServerOptions {
                    fn from(value: SvcServerOptions) -> Self {
                        let all: Vec<SvcServerOptions> = vec![SvcServerOptions { svc_url: value.svc_url.clone() }; 2];
                        Self { url: value.svc_url }
                    }
                }
            },
        );
    }

    #[test]
    fn a_struct_pattern_of_a_tag_is_prefixed_like_a_struct_literal() {
        assert_rewrites(
            parse_quote! {
                impl From<Prefixed> for Base {
                    fn from(Prefixed { url, .. }: Prefixed) -> Self {
                        let Prefixed { url: other, .. } = Prefixed::default();
                        Self { url }
                    }
                }
            },
            parse_quote! {
                impl From<SvcServerOptions> for ServerOptions {
                    fn from(SvcServerOptions { svc_url: url, .. }: SvcServerOptions) -> Self {
                        let SvcServerOptions { svc_url: other, .. } = SvcServerOptions::default();
                        Self { url }
                    }
                }
            },
        );
    }

    #[test]
    fn a_reference_param_binds_like_a_by_value_param() {
        assert_rewrites(
            parse_quote! {
                impl From<&Prefixed> for Base {
                    fn from(value: &Prefixed) -> Self {
                        Self { url: value.url.clone() }
                    }
                }
            },
            parse_quote! {
                impl From<&SvcServerOptions> for ServerOptions {
                    fn from(value: &SvcServerOptions) -> Self {
                        Self { url: value.svc_url.clone() }
                    }
                }
            },
        );
    }

    #[test]
    fn a_renamed_shorthand_field_keeps_reading_its_local() {
        assert_rewrites(
            parse_quote! {
                impl From<Base> for Prefixed {
                    fn from(value: Base) -> Self {
                        let url = value.url;
                        Self { url }
                    }
                }
            },
            parse_quote! {
                impl From<ServerOptions> for SvcServerOptions {
                    fn from(value: ServerOptions) -> Self {
                        let url = value.url;
                        Self { svc_url: url }
                    }
                }
            },
        );
    }

    #[test]
    fn a_generic_struct_keeps_its_generic_args() {
        assert_rewrites(
            parse_quote! {
                impl<T> From<Prefixed<T>> for Base<T> {
                    fn from(value: Prefixed<T>) -> Self {
                        Self { url: value.url, extra: Base::<T>::empty() }
                    }
                }
            },
            parse_quote! {
                impl<T> From<SvcServerOptions<T>> for ServerOptions<T> {
                    fn from(value: SvcServerOptions<T>) -> Self {
                        Self { url: value.svc_url, extra: ServerOptions::<T>::empty() }
                    }
                }
            },
        );
    }

    #[test]
    fn a_self_or_smart_pointer_param_binds_like_a_reference() {
        assert_rewrites(
            parse_quote! {
                impl PartialEq for Prefixed {
                    fn eq(&self, other: &Self) -> bool { self.url == other.url }
                }
            },
            parse_quote! {
                impl PartialEq for SvcServerOptions {
                    fn eq(&self, other: &Self) -> bool { self.svc_url == other.svc_url }
                }
            },
        );
        assert_rewrites(
            parse_quote! {
                impl From<Box<Prefixed>> for Base {
                    fn from(value: Box<Prefixed>) -> Self { Self { url: value.url } }
                }
            },
            parse_quote! {
                impl From<Box<SvcServerOptions>> for ServerOptions {
                    fn from(value: Box<SvcServerOptions>) -> Self { Self { url: value.svc_url } }
                }
            },
        );
    }

    #[test]
    fn a_wrapped_or_copied_receiver_is_prefixed() {
        assert_rewrites(
            parse_quote! {
                impl From<Prefixed> for Base {
                    fn from(value: Prefixed) -> Self {
                        let copy = value.clone();
                        let other = &value;
                        Self { url: format!("{}{}{}", (*other).url, (value).url, copy.url) }
                    }
                }
            },
            parse_quote! {
                impl From<SvcServerOptions> for ServerOptions {
                    fn from(value: SvcServerOptions) -> Self {
                        let copy = value.clone();
                        let other = &value;
                        Self { url: format!("{}{}{}", (*other).svc_url, (value).svc_url, copy.svc_url) }
                    }
                }
            },
        );
    }

    #[test]
    fn a_tuple_or_array_pattern_binds_each_element() {
        assert_rewrites(
            parse_quote! {
                impl From<(Prefixed, u8)> for Base {
                    fn from((value, n): (Prefixed, u8)) -> Self {
                        let [first, ..]: [Prefixed; 2] = build();
                        Self { url: value.url + &first.url }
                    }
                }
            },
            parse_quote! {
                impl From<(SvcServerOptions, u8)> for ServerOptions {
                    fn from((value, n): (SvcServerOptions, u8)) -> Self {
                        let [first, ..]: [SvcServerOptions; 2] = build();
                        Self { url: value.svc_url + &first.svc_url }
                    }
                }
            },
        );
    }

    #[test]
    fn a_nested_fn_or_impl_binds_its_own_names() {
        assert_rewrites(
            parse_quote! {
                impl Prefixed {
                    fn from_parts(value: Prefixed) -> Self {
                        fn fallback(value: &Fallback) -> String { value.url.clone() }
                        fn url_of(v: &Prefixed) -> String { v.url.clone() }
                        struct Label { url: String }
                        impl Label { fn new(url: String) -> Self { Self { url } } }
                        Self { url: url_of(&value) }
                    }
                }
            },
            parse_quote! {
                impl SvcServerOptions {
                    fn from_parts(value: SvcServerOptions) -> Self {
                        fn fallback(value: &Fallback) -> String { value.url.clone() }
                        fn url_of(v: &SvcServerOptions) -> String { v.svc_url.clone() }
                        struct Label { url: String }
                        impl Label { fn new(url: String) -> Self { Self { url } } }
                        Self { svc_url: url_of(&value) }
                    }
                }
            },
        );
    }

    #[test]
    fn an_attribute_arg_and_an_assoc_type_literal_are_prefixed() {
        assert_rewrites(
            parse_quote! {
                impl Mul<u8> for Prefixed {
                    type Output = Prefixed;

                    #[instrument(fields(url = %self.url))]
                    #[my_attr(Prefixed)]
                    fn mul(self, n: u8) -> Self::Output {
                        Self::Output { url: self.url.repeat(n.into()) }
                    }
                }
            },
            parse_quote! {
                impl Mul<u8> for SvcServerOptions {
                    type Output = SvcServerOptions;

                    #[instrument(fields(url = %self.svc_url))]
                    #[my_attr(SvcServerOptions)]
                    fn mul(self, n: u8) -> Self::Output {
                        Self::Output { svc_url: self.svc_url.repeat(n.into()) }
                    }
                }
            },
        );
    }
}
