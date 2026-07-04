use attrs::Attributes;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Item, Result};

use crate::clappen::{arm::Arm, expansion::Expansion};
use crate::helper::PrefixValue;

mod arm;
pub(crate) mod attrs;
mod expansion;

// Constant
const EXPORT_ATTR: &str = "export";

pub(crate) fn expand(
    args: TokenStream,
    attrs: Attributes,
    items: Vec<Item>,
) -> Result<TokenStream> {
    let export_macro = attrs.export;

    let unknown_items: Vec<_> = items
        .iter()
        .flat_map(|e| match e {
            Item::Impl(_) | Item::Struct(_) | Item::Use(_) => None,
            e => Some(e),
        })
        .collect();

    if !unknown_items.is_empty() {
        return Err(syn::Error::new_spanned(
            &args,
            "clappen support is limited to a single struct with one or more impl/use blocks",
        ));
    }

    let use_items: Vec<_> = items
        .iter()
        .flat_map(|e| match e {
            Item::Use(item) => Some(item),
            _ => None,
        })
        .collect();

    let struct_defs: Vec<_> = items
        .iter()
        .flat_map(|e| match e {
            Item::Struct(item) => Some(item),
            _ => None,
        })
        .collect();

    let [struct_def] = struct_defs.as_slice() else {
        return Err(syn::Error::new_spanned(
            &args,
            "clappen must have a unique struct definition",
        ));
    };

    let items_impl: Vec<_> = items
        .iter()
        .flat_map(|e| match e {
            Item::Impl(item) => Some(item),
            _ => None,
        })
        .collect();

    // struct field idents, forwarded to the prefixing macros
    let fields: Vec<_> = struct_def.fields.iter().flat_map(|e| &e.ident).collect();

    // split regular vs marked impls and build the per-prefix template arms
    let Expansion {
        regular_impls,
        base,
        prefixed,
        template,
    } = expansion::build(&items_impl, struct_def, &fields, &attrs.default_prefix)?;

    // the struct and its regular impls, for every arm but `@__template`
    let arm_body = |arm: Arm| {
        let prefix_arg = arm.attribute_prefix();
        let default_prefix = attrs.default_prefix.value().as_slice();
        let item_impls = regular_impls.iter().map(|e| {
            quote! {
                #[clappen::__clappen_impl(#prefix_arg #(default_prefix = #default_prefix,)* prefixed_fields = [#(#fields),*])]
                #e
            }
        });

        quote! {
            #(#use_items)*
            #[clappen::__clappen_struct(#prefix_arg #(default_prefix = #default_prefix,)*)]
            #struct_def
            #(#item_impls)*
        }
    };
    let base_struct = arm_body(Arm::Base);
    let prefixed_struct = arm_body(Arm::Prefixed);
    let nested_struct = arm_body(Arm::Struct);

    let macro_doc = " Invoke with `()` for the base struct, or `(\"prefix\")` for a prefixed copy. The `@__`-prefixed forms are internal and not part of the public API.";

    let base_child_apply = &base.child_apply.invocations;
    let prefixed_self_apply = &prefixed.self_apply.impls;
    let prefixed_child_apply = &prefixed.child_apply.invocations;
    let template_self_apply = &template.self_apply.impls;
    let template_child_apply = &template.child_apply.invocations;

    Ok(quote! {
        #[doc = #macro_doc]
        #[macro_export]
        macro_rules! #export_macro {
            // base: the plain struct, and its children's conversions if it has no template
            () => {
                #base_struct
                #(#base_child_apply)*
            };
            // prefixed: the struct, its conversion and its children's. It does not call its own
            // `@__template`, because that call does not resolve across crates
            ($prefix: literal) => {
                #prefixed_struct
                #(#prefixed_self_apply)*
                #(#prefixed_child_apply)*
            };

            // internal arms, not public API
            // `@__struct`: a nested field's type, for `clappen_command`
            (@__struct $($prefix: literal)?) => {
                #nested_struct
            };
            // `@__template`: a parent's `child_apply`, the prefixed arm's output at its chain
            (@__template $($prefix: literal,)? chain = [
                $( ( $($command_prefix: literal,)? $field: ident $(, $parent_default: literal)? ) ),* $(,)?
            ]) => {
                #(#template_self_apply)*
                #(#template_child_apply)*
            };
        }
    })
}
