// Codegen for `#[clappen_template_impl]`: split the impls tagged with it from the regular ones and
// build the per-prefix pieces inserted into the exported macro (see `clappen::create_template`).

use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{Ident, ItemImpl, ItemStruct, Path, Token};

use crate::clappen_command::attrs::{Attributes as CommandAttributes, NestedAttributes};
use crate::clappen_struct::FIELD_ATTR_CLAPPEN_COMMAND;
use crate::clappen_template_impl::{IMPL_ATTR_CLAPPEN_TEMPLATE, attrs::TemplateTags};

// the per-prefix pieces `build` returns; `clappen::create_template` inserts them into the exported macro
pub(crate) struct Expansion {
    // impls without the `#[clappen_template_impl]` marker, handled like any clappen impl
    pub(crate) regular_impls: Vec<ItemImpl>,
    // `($prefix)` arm: this struct's own template impl, specialized for the prefix
    pub(crate) prefixed_self_apply: Vec<TokenStream>,
    // `($prefix)` arm: recurse into each flattened child by calling its macro
    pub(crate) prefixed_child_apply: Vec<TokenStream>,
    // `@__template` arm (this struct flattened in a parent): `self_apply` at the inherited chain
    pub(crate) chained_self_apply: Vec<TokenStream>,
    // `@__template` arm: `child_apply`, extending the inherited chain
    pub(crate) chained_child_apply: Vec<TokenStream>,
    // `()` arm: `child_apply` with an empty prefix, only when this struct has no template of its own
    pub(crate) base_child_apply: Vec<TokenStream>,
}

// a template-marked impl paired with its optional `Base`/`Prefixed` tag overrides
struct TemplateImpl {
    item: ItemImpl,
    tags: TemplateTags,
}

pub(crate) fn build(
    items_impl: &[&ItemImpl],
    struct_def: &ItemStruct,
    fields: &[TokenStream],
    default_prefix: &str,
) -> syn::Result<Expansion> {
    // split regular vs template impls; the marker may override the `Base`/`Prefixed` tag idents
    // (left as `None` here and defaulted by the proc-macro)
    let mut regular_impls = Vec::new();
    let mut template_impls: Vec<TemplateImpl> = Vec::new();
    for item in items_impl {
        let mut item = (*item).clone();
        let marker_pos = item
            .attrs
            .iter()
            .position(|attr| attr.path().is_ident(IMPL_ATTR_CLAPPEN_TEMPLATE));
        if let Some(pos) = marker_pos {
            let tags = TemplateTags::try_from(&item.attrs.remove(pos))?;
            template_impls.push(TemplateImpl { item, tags });
        } else {
            regular_impls.push(item);
        }
    }

    let struct_ident = &struct_def.ident;
    let nested_fields = collect_nested_fields(struct_def);

    // `self_apply` is this struct's own template impl, tagged for the proc-macro to rewrite later
    // (the same idea as `#[__clappen_impl]`). `child_apply` recurses into each flattened child by
    // calling its macro. Both are built once here, parameterized by the prefix and the chain, then
    // reused across the arms: the arms can't call one another, because a `#[macro_export]` macro
    // built by a proc-macro can't name itself across crates.
    let apply_pieces = |prefix: TokenStream, chain: TokenStream| {
        let self_apply: Vec<TokenStream> = template_impls
            .iter()
            .map(|TemplateImpl { item, tags }| {
                quote! {
                    #[clappen::__clappen_template_impl(prefix = #prefix, default_prefix = #default_prefix, struct_ident = #struct_ident, #tags chain = [ #chain ], prefixed_fields = [#(#fields)*])]
                    #item
                }
            })
            .collect();
        let child_apply: Vec<TokenStream> = nested_fields
            .iter()
            .map(|(apply, command_prefix, field_ident)| {
                quote! {
                    #apply!(@__template #prefix, chain = [ #chain (#command_prefix, #field_ident, #default_prefix) ]);
                }
            })
            .collect();
        (self_apply, child_apply)
    };

    // `@__template` arm: reached when this struct is flattened in a parent; the chain is the metavars
    // the arm matched
    let (chained_self_apply, chained_child_apply) = apply_pieces(
        quote! { $prefix },
        quote! { $( ($command_prefix, $field, $parent_default), )* },
    );

    // `($prefix)` arm: top-level prefixed instantiation, so the chain starts empty
    let (prefixed_self_apply, prefixed_child_apply) = apply_pieces(quote! { $prefix }, quote! {});

    // `()` arm: a template-free parent still converts each flattened child to its base (empty
    // prefix). A parent with its own template gets the children's conversions from its prefixed
    // instantiations instead, so this is empty then.
    let base_child_apply = if template_impls.is_empty() {
        let (_, child_apply) = apply_pieces(quote! { "" }, quote! {});
        child_apply
    } else {
        Vec::new()
    };

    Ok(Expansion {
        regular_impls,
        prefixed_self_apply,
        prefixed_child_apply,
        chained_self_apply,
        chained_child_apply,
        base_child_apply,
    })
}

// flattened fields used to build the child template calls: (apply macro, command prefix, field name)
fn collect_nested_fields(struct_def: &ItemStruct) -> Vec<(Path, String, Ident)> {
    struct_def
        .fields
        .iter()
        .filter_map(|field| {
            let field_ident = field.ident.clone()?;
            let attr = field
                .attrs
                .iter()
                .find(|a| a.path().is_ident(FIELD_ATTR_CLAPPEN_COMMAND))?;
            let metas = attr
                .parse_args_with(Punctuated::<NestedAttributes, Token![,]>::parse_terminated)
                .ok()?;
            let cmd: CommandAttributes = metas.into_iter().collect::<Vec<_>>().try_into().ok()?;
            Some((cmd.apply, cmd.prefix, field_ident))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::{ToTokens, quote};
    use syn::parse_quote;

    // (apply macro path, command prefix, field ident) as strings, for readable assertions
    fn collected(struct_def: &ItemStruct) -> Vec<(String, String, String)> {
        collect_nested_fields(struct_def)
            .into_iter()
            .map(|(apply, prefix, field)| {
                (
                    apply.to_token_stream().to_string(),
                    prefix,
                    field.to_string(),
                )
            })
            .collect()
    }

    #[test]
    fn collect_nested_fields_keeps_declaration_order() {
        let struct_def: ItemStruct = parse_quote! {
            struct App {
                name: String,
                #[clappen_command(apply = db, prefix = "db")]
                database: Db,
                #[clappen_command(apply = cache, prefix = "cache")]
                cache: Cache,
            }
        };
        assert_eq!(
            collected(&struct_def),
            vec![
                ("db".to_string(), "db".to_string(), "database".to_string()),
                (
                    "cache".to_string(),
                    "cache".to_string(),
                    "cache".to_string()
                ),
            ],
        );
    }

    #[test]
    fn collect_nested_fields_defaults_prefix_to_empty() {
        let struct_def: ItemStruct = parse_quote! {
            struct App {
                #[clappen_command(apply = db)]
                database: Db,
            }
        };
        assert_eq!(
            collected(&struct_def),
            vec![("db".to_string(), String::new(), "database".to_string())],
        );
    }

    // apply may be a module/crate-qualified path (this is what makes cross-crate flattening
    // resolve); the whole path is kept, not just the last segment
    #[test]
    fn collect_nested_fields_preserves_qualified_apply_path() {
        let struct_def: ItemStruct = parse_quote! {
            struct App {
                #[clappen_command(apply = pools::pool, prefix = "pool")]
                conn: Pool,
            }
        };
        assert_eq!(
            collected(&struct_def),
            vec![(
                "pools :: pool".to_string(),
                "pool".to_string(),
                "conn".to_string()
            )],
        );
    }

    // the `&[&ItemImpl]` slice `build` takes
    fn impl_refs(impls: &[ItemImpl]) -> Vec<&ItemImpl> {
        impls.iter().collect()
    }

    #[test]
    fn build_separates_regular_and_template_impls() {
        let struct_def: ItemStruct = parse_quote! {
            struct ServerOptions { url: String }
        };
        let impls: Vec<ItemImpl> = vec![
            parse_quote! {
                impl ServerOptions {
                    fn url(&self) -> &str { &self.url }
                }
            },
            parse_quote! {
                #[clappen_template_impl]
                impl From<Prefixed> for Base {
                    fn from(value: Prefixed) -> Self { Self { url: value.url } }
                }
            },
        ];
        let fields = vec![quote! { url }];

        let expansion = build(&impl_refs(&impls), &struct_def, &fields, "").unwrap();

        // the plain impl stays regular; the marked one turns into self_apply
        assert_eq!(expansion.regular_impls.len(), 1);
        assert_eq!(expansion.prefixed_self_apply.len(), 1);
        assert_eq!(expansion.chained_self_apply.len(), 1);
        // no flattened fields, so no child recursion in any arm
        assert!(expansion.prefixed_child_apply.is_empty());
        assert!(expansion.chained_child_apply.is_empty());
        // this struct has its own template, so the base arm emits nothing
        assert!(expansion.base_child_apply.is_empty());
    }

    #[test]
    fn build_without_template_emits_base_child_apply() {
        let struct_def: ItemStruct = parse_quote! {
            struct App {
                #[clappen_command(apply = db, prefix = "db")]
                database: Db,
            }
        };
        let impls: Vec<ItemImpl> = vec![];
        let fields = vec![quote! { database }];

        let expansion = build(&impl_refs(&impls), &struct_def, &fields, "").unwrap();

        // no template impls, so nothing to specialize for self
        assert!(expansion.prefixed_self_apply.is_empty());
        assert!(expansion.chained_self_apply.is_empty());
        // one flattened field -> one child recursion per arm, including the base arm
        assert_eq!(expansion.prefixed_child_apply.len(), 1);
        assert_eq!(expansion.chained_child_apply.len(), 1);
        assert_eq!(expansion.base_child_apply.len(), 1);

        // the base arm's recursion calls the child's `@__template` for the `database` field,
        // at an empty prefix with the one-step chain
        let expected = quote! {
            db!(@__template "", chain = [("db", database, "")]);
        };
        assert_eq!(
            expansion.base_child_apply[0].to_string(),
            expected.to_string()
        );
    }

    #[test]
    fn build_self_apply_forwards_expected_attributes() {
        let struct_def: ItemStruct = parse_quote! {
            struct ServerOptions { url: String }
        };
        let impls: Vec<ItemImpl> = vec![parse_quote! {
            #[clappen_template_impl]
            impl From<Prefixed> for Base {
                fn from(value: Prefixed) -> Self { Self { url: value.url } }
            }
        }];
        let fields = vec![quote! { url }];

        let expansion = build(&impl_refs(&impls), &struct_def, &fields, "svc").unwrap();

        // the marker is stripped and prefix/default_prefix/struct_ident/chain/prefixed_fields are
        // forwarded to the internal proc-macro, with `$prefix` left for the macro arm to fill
        let expected = quote! {
            #[clappen::__clappen_template_impl(prefix = $prefix, default_prefix = "svc", struct_ident = ServerOptions, chain = [], prefixed_fields = [url])]
            impl From<Prefixed> for Base {
                fn from(value: Prefixed) -> Self {
                    Self { url: value.url }
                }
            }
        };
        assert_eq!(
            expansion.prefixed_self_apply[0].to_string(),
            expected.to_string()
        );
    }
}
