// Codegen for `#[clappen_template_impl]`: split the impls tagged with it from the regular ones and
// build the per-prefix pieces inserted into the exported macro (see `clappen::create_template`).

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, ItemImpl, ItemStruct};

use crate::clappen_command::attrs::Attributes as CommandAttributes;
use crate::clappen_struct::FIELD_ATTR_CLAPPEN_COMMAND;
use crate::clappen_template_impl::{
    IMPL_ATTR_CLAPPEN_TEMPLATE,
    attrs::{ChainStepTokens, TemplateTags},
};
use crate::helper::{
    PrefixValue,
    prefix::{CommandPrefix, DefaultPrefix},
};

// which arm of the exported `macro_rules!` these pieces are spliced into
#[derive(Clone, Copy)]
enum Arm {
    // `NAME!()`
    Base,
    // `NAME!("p")`
    Prefixed,
    // `NAME!(@__template "p", chain = [ .. ])`
    Chained,
}

// how an arm's macro pattern binds its own prefix
enum ArmPrefix {
    Absent,
    // `($prefix: literal)`
    Required,
    // `($($prefix: literal,)? ..)`, so the caller may skip it
    Optional,
}

impl ArmPrefix {
    // the `prefix = ..` key the internal attribute takes
    fn attribute(&self) -> TokenStream {
        match self {
            ArmPrefix::Absent => TokenStream::new(),
            ArmPrefix::Required => quote! { prefix = $prefix, },
            ArmPrefix::Optional => quote! { $(prefix = $prefix,)? },
        }
    }

    // the leading slot of a nested call
    fn invocation(&self) -> TokenStream {
        match self {
            ArmPrefix::Absent => TokenStream::new(),
            ArmPrefix::Required => quote! { $prefix, },
            ArmPrefix::Optional => quote! { $($prefix,)? },
        }
    }
}

impl Arm {
    fn prefix(self) -> ArmPrefix {
        match self {
            Arm::Base => ArmPrefix::Absent,
            Arm::Prefixed => ArmPrefix::Required,
            Arm::Chained => ArmPrefix::Optional,
        }
    }

    // the steps an arm forwards: only a chained arm has any, and it passes on what it matched
    fn chain(self) -> TokenStream {
        match self {
            Arm::Base | Arm::Prefixed => TokenStream::new(),
            // a repetition is not a value, so the `$( ),*` wrapper stays here; the step inside it
            // comes from the same writer the concrete calls use
            Arm::Chained => {
                let step = ChainStepTokens::Forwarded;
                quote! { $( #step, )* }
            }
        }
    }

    // which halves this arm emits: this is ARCHITECTURE.md section 4's table. The base is
    // unprefixed, so it has no prefixed type to convert from; and a parent with its own template
    // gets its children's conversions from its prefixed instantiations instead.
    fn applies(self, has_own_template: bool) -> ApplyRule {
        match (self, has_own_template) {
            (Arm::Base, true) => ApplyRule::Nothing,
            (Arm::Base, false) => ApplyRule::ChildOnly,
            (Arm::Prefixed | Arm::Chained, _) => ApplyRule::Both,
        }
    }
}

// the two halves of what an arm emits, wrapped so one cannot be assigned where the other belongs.
// This struct's own template impls, tagged for `__clappen_template_impl`:
pub(crate) struct SelfApply {
    pub(crate) impls: Vec<TokenStream>,
}
// and one `CHILD!(@__template ..)` invocation per flattened field:
pub(crate) struct ChildApply {
    pub(crate) invocations: Vec<TokenStream>,
}

// what one arm emits for the template feature
pub(crate) struct Apply {
    pub(crate) self_apply: SelfApply,
    pub(crate) child_apply: ChildApply,
}

// which halves an arm emits. Three states, not four: an arm never emits its own conversion
// without also recursing into its children.
enum ApplyRule {
    Nothing,
    ChildOnly,
    Both,
}

// the per-arm pieces `build` returns; `clappen::create_template` inserts them into the exported macro
pub(crate) struct Expansion {
    // impls without the `#[clappen_template_impl]` marker, handled like any clappen impl
    pub(crate) regular_impls: Vec<ItemImpl>,
    // the base struct is unprefixed, so it has no prefixed type to convert from, and therefore no
    // `SelfApply` to misuse
    pub(crate) base_child_apply: ChildApply,
    pub(crate) prefixed: Apply,
    pub(crate) chained: Apply,
}

// a template-marked impl paired with its optional `Base`/`Prefixed` tag overrides
struct TemplateImpl {
    item: ItemImpl,
    tags: TemplateTags,
}

pub(crate) fn build(
    items_impl: &[&ItemImpl],
    struct_def: &ItemStruct,
    fields: &[&Ident],
    default_prefix: &DefaultPrefix,
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
    let nested_fields = collect_nested_fields(struct_def)?;

    // `own` is this struct's own template impl, tagged for the proc-macro to rewrite later (the same
    // idea as `#[__clappen_impl]`). `children` recurses into each flattened child by calling its
    // macro. Both are built per arm rather than shared: the arms can't call one another, because a
    // `#[macro_export]` macro built by a proc-macro can't name itself across crates.
    let has_own_template = !template_impls.is_empty();
    // the internal attribute's key, dropped when this struct has no default prefix. The chain step
    // a child reads as its parent's default prefix is filled by `ChainStepTokens`.
    let default_prefix_attr = default_prefix
        .value()
        .as_ref()
        .map(|e| quote! { default_prefix = #e, });
    let apply = |arm: Arm| -> Apply {
        let attribute_prefix = arm.prefix().attribute();
        let invocation_prefix = arm.prefix().invocation();
        let chain = arm.chain();
        // an arm that emits neither gets an empty source, so both chains below stay flat
        let (templates, nested): (&[TemplateImpl], &[NestedField]) =
            match arm.applies(has_own_template) {
                ApplyRule::Nothing => (&[], &[]),
                ApplyRule::ChildOnly => (&[], &nested_fields),
                ApplyRule::Both => (&template_impls, &nested_fields),
            };
        Apply {
            self_apply: SelfApply {
                impls:                 templates
                    .iter()
                    .map(|TemplateImpl { item, tags }| {
                        quote! {
                            #[clappen::__clappen_template_impl(#attribute_prefix #default_prefix_attr struct_ident = #struct_ident, #tags chain = [ #chain ], prefixed_fields = [#(#fields),*])]
                            #item
                        }
                    })
                    .collect(),
            },
            child_apply: ChildApply {
                invocations:                 nested
                    .iter()
                    .map(|(apply, command_prefix, field_ident)| {
                        let step = ChainStepTokens::Concrete {
                            command_prefix,
                            field: field_ident,
                            parent_default: default_prefix,
                        };
                        quote! {
                            #apply!(@__template #invocation_prefix chain = [ #chain #step ]);
                        }
                    })
                    .collect(),
            },
        }
    };

    Ok(Expansion {
        regular_impls,
        // the base arm's `SelfApply` is always empty, so only the child half is kept
        base_child_apply: apply(Arm::Base).child_apply,
        prefixed: apply(Arm::Prefixed),
        chained: apply(Arm::Chained),
    })
}

// a flattened field: (apply macro, the field's command prefix, field name)
type NestedField = (TokenStream, CommandPrefix, Ident);

// flattened fields used to build the child template calls.
// A field without `#[clappen_command]` is skipped; a field that has one but does not parse is an
// error, so a bad attribute cannot silently cost the child its conversion.
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
        nested.push((cmd.apply, cmd.prefix, field_ident));
    }
    Ok(nested)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::{format_ident, quote};
    use syn::parse_quote;

    use crate::helper::parse_literal;

    // (apply macro path, command prefix, field ident) as strings, for readable assertions
    fn collected(struct_def: &ItemStruct) -> Vec<(String, String, String)> {
        collect_nested_fields(struct_def)
            .unwrap()
            .into_iter()
            .map(|(apply, prefix, field)| {
                (
                    apply.to_string(),
                    prefix.value().clone().unwrap_or_default(),
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
    fn collect_nested_fields_omits_an_absent_prefix() {
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
        // (Base, has a template): emits nothing at all
        assert!(expansion.base_child_apply.invocations.is_empty());
        // Prefixed and Chained: one `self_apply`, and no `child_apply` since nothing is flattened
        assert_eq!(expansion.prefixed.self_apply.impls.len(), 1);
        assert!(expansion.prefixed.child_apply.invocations.is_empty());
        assert_eq!(expansion.chained.self_apply.impls.len(), 1);
        assert!(expansion.chained.child_apply.invocations.is_empty());
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
        let database = format_ident!("database");
        let fields = vec![&database];

        let expansion = build(
            &impl_refs(&impls),
            &struct_def,
            &fields,
            &DefaultPrefix::default(),
        )
        .unwrap();

        // no template impls, so no arm has a `self_apply`
        assert!(expansion.prefixed.self_apply.impls.is_empty());
        assert!(expansion.chained.self_apply.impls.is_empty());
        // (Base, no template): the one flattened field recurses in every arm, base included
        assert_eq!(expansion.base_child_apply.invocations.len(), 1);
        assert_eq!(expansion.prefixed.child_apply.invocations.len(), 1);
        assert_eq!(expansion.chained.child_apply.invocations.len(), 1);

        // the base arm's recursion calls the child's `@__template` for the `database` field,
        // with no prefix of its own and the one-step chain
        let expected = quote! {
            db!(@__template chain = [("db", database)]);
        };
        assert_eq!(
            expansion.base_child_apply.invocations[0].to_string(),
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
        let url = format_ident!("url");
        let fields = vec![&url];

        let expansion = build(
            &impl_refs(&impls),
            &struct_def,
            &fields,
            &parse_literal("svc"),
        )
        .unwrap();

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
            expansion.prefixed.self_apply.impls[0].to_string(),
            expected.to_string()
        );
    }

    // the `(Arm::Base, true)` row: a struct with BOTH a template and a flattened field. The base arm
    // stays empty because the children's conversions come from the prefixed instantiations instead.
    #[test]
    fn build_with_template_and_flattened_field_leaves_the_base_arm_empty() {
        let struct_def: ItemStruct = parse_quote! {
            struct App {
                url: String,
                #[clappen_command(apply = db, prefix = "db")]
                database: Db,
            }
        };
        let impls: Vec<ItemImpl> = vec![parse_quote! {
            #[clappen_template_impl]
            impl From<Prefixed> for Base {
                fn from(value: Prefixed) -> Self {
                    Self { url: value.url, database: value.database.into() }
                }
            }
        }];
        let (url, database) = (format_ident!("url"), format_ident!("database"));
        let fields = vec![&url, &database];

        let expansion = build(
            &impl_refs(&impls),
            &struct_def,
            &fields,
            &DefaultPrefix::default(),
        )
        .unwrap();

        assert!(expansion.base_child_apply.invocations.is_empty());
        assert_eq!(expansion.prefixed.self_apply.impls.len(), 1);
        assert_eq!(expansion.prefixed.child_apply.invocations.len(), 1);
        assert_eq!(expansion.chained.self_apply.impls.len(), 1);
        assert_eq!(expansion.chained.child_apply.invocations.len(), 1);
    }
}
