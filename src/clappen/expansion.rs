// Splits the marked impls from the regular ones and builds the template pieces of each macro arm.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, ItemImpl, ItemStruct};

use crate::clappen::arm::Arm;
use crate::clappen_command::{
    FIELD_ATTR_CLAPPEN_COMMAND,
    attrs::{ApplyPath, Attributes as CommandAttributes},
};
use crate::clappen_template_impl::{IMPL_ATTR_CLAPPEN_TEMPLATE, attrs::TemplateTags};
use crate::helper::{
    PrefixValue,
    prefix::{CommandPrefix, DefaultPrefix},
};

// this struct's own template impls, tagged for `__clappen_template_impl`
pub(crate) struct SelfApply {
    pub impls: Vec<TokenStream>,
}
// one `CHILD!(@__template ..)` call per flattened field
pub(crate) struct ChildApply {
    pub invocations: Vec<TokenStream>,
}

// `SelfApply`, or `()` for the base arm
pub(crate) trait SelfApplySlot {}
impl SelfApplySlot for SelfApply {}
impl SelfApplySlot for () {}

// what one arm emits for the template feature
pub(crate) struct Apply<S: SelfApplySlot = SelfApply> {
    pub self_apply: S,
    pub child_apply: ChildApply,
}

// what `build` returns, for `clappen::expand` to put in the exported macro
pub(crate) struct Expansion {
    // impls without the marker, handled like any clappen impl
    pub regular_impls: Vec<ItemImpl>,
    // no `SelfApply`: the base struct has no prefixed type to convert from
    pub base: Apply<()>,
    pub prefixed: Apply,
    pub template: Apply,
}

// a marked impl and its tag overrides
struct TemplateImpl {
    item: ItemImpl,
    tags: TemplateTags,
}

// a flattened field: its apply macro, its command prefix and its name
struct NestedField {
    apply: ApplyPath,
    command_prefix: CommandPrefix,
    field: Ident,
}

pub(crate) fn build(
    items_impl: &[&ItemImpl],
    struct_def: &ItemStruct,
    fields: &[&Ident],
    default_prefix: &DefaultPrefix,
) -> syn::Result<Expansion> {
    // a tag the marker does not override stays `None`; the proc-macro fills in the default
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
    let nested_fields = collect_nested_fields(struct_def)?;

    // built per arm, not shared: an arm cannot call another one, because a `#[macro_export]` macro
    // built by a proc-macro cannot name itself across crates
    let child_apply = |arm: Arm| -> ChildApply {
        let invocation_prefix = arm.invocation_prefix();
        let chain = arm.chain();
        let parent_default = default_prefix.value().as_slice();
        ChildApply {
            invocations: nested_fields
                .iter()
                .map(|nested| {
                    let apply = &nested.apply;
                    let field = &nested.field;
                    let command_prefix = nested.command_prefix.value().iter();

                    quote! {
                        #apply!(@__template #invocation_prefix chain = [
                            #chain ( #(#command_prefix,)* #field #(, #parent_default)* )
                        ]);
                    }
                })
                .collect(),
        }
    };
    let apply = |arm: Arm| -> Apply {
        let attribute_prefix = arm.attribute_prefix();
        let chain = arm.chain();
        let default_prefix = default_prefix.value().as_slice();
        Apply {
            self_apply: SelfApply {
                impls: template_impls
                    .iter()
                    .map(|TemplateImpl { item, tags }| {
                        quote! {
                            #[clappen::__clappen_template_impl(
                                #attribute_prefix #(default_prefix = #default_prefix,)*
                                struct_ident = #struct_ident, #tags chain = [ #chain ],
                                prefixed_fields = [#(#fields),*]
                            )]
                            #item
                        }
                    })
                    .collect(),
            },
            child_apply: child_apply(arm),
        }
    };

    Ok(Expansion {
        regular_impls,
        base: Apply {
            self_apply: (),
            // a parent with a template gets its children's conversions in its prefixed arms
            child_apply: if template_impls.is_empty() {
                child_apply(Arm::Base)
            } else {
                ChildApply {
                    invocations: Vec::new(),
                }
            },
        },
        prefixed: apply(Arm::Prefixed),
        template: apply(Arm::Template),
    })
}

// the `#[clappen_command]` fields. One that does not parse is an error, so a child never loses its
// conversion without a message.
fn collect_nested_fields(struct_def: &ItemStruct) -> syn::Result<Vec<NestedField>> {
    let mut nested = Vec::new();
    for field in &struct_def.fields {
        let Some(attr) = field
            .attrs
            .iter()
            .find(|a| a.path().is_ident(FIELD_ATTR_CLAPPEN_COMMAND))
        else {
            continue;
        };
        let Some(field_ident) = field.ident.clone() else {
            continue;
        };
        let cmd: CommandAttributes = attr.parse_args()?;
        nested.push(NestedField {
            apply: cmd.apply,
            command_prefix: cmd.prefix,
            field: field_ident,
        });
    }
    Ok(nested)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::{ToTokens, format_ident, quote};
    use syn::parse_quote;

    use crate::helper::parse_literal;

    // (apply macro path, command prefix, field ident) as strings, for readable assertions
    fn collected(struct_def: &ItemStruct) -> Vec<(String, String, String)> {
        collect_nested_fields(struct_def)
            .unwrap()
            .into_iter()
            .map(|nested| {
                (
                    nested.apply.to_token_stream().to_string(),
                    nested.command_prefix.as_str().to_string(),
                    nested.field.to_string(),
                )
            })
            .collect()
    }

    #[test]
    fn collect_nested_fields_keeps_declaration_order() {
        let struct_def: ItemStruct = parse_quote! {
            struct ServerOptions {
                name: String,
                #[clappen_command(apply = nested, prefix = "db")]
                nested: MyStruct,
                #[clappen_command(apply = nested, prefix = "cache")]
                nested1: MyStruct,
            }
        };
        assert_eq!(
            collected(&struct_def),
            vec![
                ("nested".to_string(), "db".to_string(), "nested".to_string()),
                (
                    "nested".to_string(),
                    "cache".to_string(),
                    "nested1".to_string()
                ),
            ],
        );
    }

    #[test]
    fn collect_nested_fields_omits_an_absent_prefix() {
        let struct_def: ItemStruct = parse_quote! {
            struct ServerOptions {
                #[clappen_command(apply = nested)]
                nested: MyStruct,
            }
        };
        assert_eq!(
            collected(&struct_def),
            vec![("nested".to_string(), String::new(), "nested".to_string())],
        );
    }

    // a qualified apply path (needed across crates) is kept whole
    #[test]
    fn collect_nested_fields_preserves_qualified_apply_path() {
        let struct_def: ItemStruct = parse_quote! {
            struct ServerOptions {
                #[clappen_command(apply = pools::nested, prefix = "pool")]
                nested: MyStruct,
            }
        };
        assert_eq!(
            collected(&struct_def),
            vec![(
                "pools :: nested".to_string(),
                "pool".to_string(),
                "nested".to_string()
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
        let url = format_ident!("url");
        let fields = vec![&url];

        let expansion = build(
            &impl_refs(&impls),
            &struct_def,
            &fields,
            &DefaultPrefix::default(),
        )
        .unwrap();

        // the plain impl stays regular; the marked one becomes each arm's `self_apply`
        assert_eq!(expansion.regular_impls.len(), 1);
        // (Base, with a template): nothing
        assert!(expansion.base.child_apply.invocations.is_empty());
        // Prefixed and Template: one `self_apply`, and no `child_apply` since nothing is flattened
        assert_eq!(expansion.prefixed.self_apply.impls.len(), 1);
        assert!(expansion.prefixed.child_apply.invocations.is_empty());
        assert_eq!(expansion.template.self_apply.impls.len(), 1);
        assert!(expansion.template.child_apply.invocations.is_empty());
    }

    #[test]
    fn build_without_template_emits_base_child_apply() {
        let struct_def: ItemStruct = parse_quote! {
            struct ServerOptions {
                #[clappen_command(apply = nested, prefix = "db")]
                nested: MyStruct,
            }
        };
        let nested = format_ident!("nested");
        let fields = vec![&nested];

        let expansion = build(&[], &struct_def, &fields, &DefaultPrefix::default()).unwrap();

        // no template impls, so no arm has a `self_apply`
        assert!(expansion.prefixed.self_apply.impls.is_empty());
        assert!(expansion.template.self_apply.impls.is_empty());
        // (Base, no template): the one flattened field recurses in every arm, base included
        assert_eq!(expansion.base.child_apply.invocations.len(), 1);
        assert_eq!(expansion.prefixed.child_apply.invocations.len(), 1);
        assert_eq!(expansion.template.child_apply.invocations.len(), 1);

        // the base arm calls the child's `@__template` with no prefix and a one-step chain
        let expected = quote! {
            nested!(@__template chain = [("db", nested)]);
        };
        assert_eq!(
            expansion.base.child_apply.invocations[0].to_string(),
            expected.to_string()
        );
    }

    #[test]
    fn build_writes_every_chain_step_shape() {
        let cases = [
            (None, None, quote! { (nested) }),
            (Some("p"), None, quote! { ("p", nested) }),
            (None, Some("d"), quote! { (nested, "d") }),
            (Some("p"), Some("d"), quote! { ("p", nested, "d") }),
        ];

        for (command_prefix, default_prefix, step) in cases {
            let command_prefix = command_prefix.iter();
            let struct_def: ItemStruct = parse_quote! {
                struct ServerOptions {
                    #[clappen_command(apply = nested #(, prefix = #command_prefix)*)]
                    nested: MyStruct,
                }
            };
            let default_prefix = default_prefix.map_or_else(DefaultPrefix::default, parse_literal);
            let nested = format_ident!("nested");

            let expansion = build(&[], &struct_def, &[&nested], &default_prefix).unwrap();

            let expected = quote! {
                nested!(@__template chain = [#step]);
            };
            assert_eq!(
                expansion.base.child_apply.invocations[0].to_string(),
                expected.to_string()
            );
        }
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
        let url = format_ident!("url");
        let fields = vec![&url];

        let expansion = build(
            &impl_refs(&impls),
            &struct_def,
            &fields,
            &parse_literal("svc"),
        )
        .unwrap();

        // the marker is gone, and `$prefix` is left for the macro arm to fill
        let expected = quote! {
            #[clappen::__clappen_template_impl(prefix = $prefix, default_prefix = "svc", struct_ident = ServerOptions, chain = [], prefixed_fields = [url])]
            impl From<Prefixed> for Base {
                fn from(value: Prefixed) -> Self {
                    Self { url: value.url }
                }
            }
        };
        assert_eq!(
            expansion.prefixed.self_apply.impls[0].to_string(),
            expected.to_string()
        );
    }

    // with a template and a flattened field, the base arm stays empty
    #[test]
    fn build_with_template_and_flattened_field_leaves_the_base_arm_empty() {
        let struct_def: ItemStruct = parse_quote! {
            struct ServerOptions {
                url: String,
                #[clappen_command(apply = nested, prefix = "db")]
                nested: MyStruct,
            }
        };
        let impls: Vec<ItemImpl> = vec![parse_quote! {
            #[clappen_template_impl]
            impl From<Prefixed> for Base {
                fn from(value: Prefixed) -> Self {
                    Self { url: value.url, nested: value.nested.into() }
                }
            }
        }];
        let (url, nested) = (format_ident!("url"), format_ident!("nested"));
        let fields = vec![&url, &nested];

        let expansion = build(
            &impl_refs(&impls),
            &struct_def,
            &fields,
            &DefaultPrefix::default(),
        )
        .unwrap();

        assert!(expansion.base.child_apply.invocations.is_empty());
        assert_eq!(expansion.prefixed.self_apply.impls.len(), 1);
        assert_eq!(expansion.prefixed.child_apply.invocations.len(), 1);
        assert_eq!(expansion.template.self_apply.impls.len(), 1);
        assert_eq!(expansion.template.child_apply.invocations.len(), 1);
    }
}
