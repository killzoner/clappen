use attrs::Attributes;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Item;

use self::expansion::{ChildApply, Expansion};
use crate::helper::PrefixValue;

pub(crate) mod attrs;
mod expansion;

pub(crate) fn create_template(
    args: TokenStream,
    attrs: Attributes,
    items: Vec<Item>,
) -> TokenStream {
    let export_macro = attrs.export;

    let default_prefix = attrs.default_prefix.value();
    // no key when the module has no default prefix
    let default_prefix_arg = default_prefix
        .as_ref()
        .map(|e| quote! { default_prefix = #e });

    let unknown_items: Vec<_> = items
        .iter()
        .flat_map(|e| match e {
            Item::Impl(_) | Item::Struct(_) | Item::Use(_) => None,
            e => Some(e),
        })
        .collect();

    if !unknown_items.is_empty() {
        return syn::Error::new_spanned(
            &args,
            "clappen support is limited to a single struct with one or more impl/use blocks",
        )
        .to_compile_error();
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

    if struct_defs.len() > 1 {
        return syn::Error::new_spanned(&args, "clappen must have a unique struct definition")
            .to_compile_error();
    }

    let struct_def = match struct_defs.first() {
        Some(e) => e,
        None => {
            return syn::Error::new_spanned(&args, "clappen must have a unique struct definition")
                .to_compile_error();
        }
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
        base_child_apply: ChildApply {
            invocations: base_child_apply,
        },
        prefixed,
        chained,
    } = match expansion::build(&items_impl, struct_def, &fields, &attrs.default_prefix) {
        Ok(expansion) => expansion,
        Err(e) => return e.to_compile_error(),
    };

    let prefixed_item_impls: Vec<_> = regular_impls
        .iter()
        .map(|e| {
            quote! {
                #[clappen::__clappen_impl(prefix = $prefix, prefixed_fields = [#(#fields),*], #default_prefix_arg)]
                #e
            }
        })
        .collect();

    let default_item_impls: Vec<_> = regular_impls
        .iter()
        .map(|e| {
            quote! {
                #[clappen::__clappen_impl(prefixed_fields = [#(#fields),*], #default_prefix_arg)]
                #e
            }
        })
        .collect();

    let default = match default_prefix {
        None => {
            quote! {
                #(#use_items)*
                #[clappen::__clappen_struct]
                #struct_def
                #(#default_item_impls)*
            }
        }
        Some(default_prefix) => {
            quote! {
                #(#use_items)*
                #[clappen::__clappen_struct(default_prefix = #default_prefix)]
                #struct_def
                #(#default_item_impls)*
            }
        }
    };

    let prefixed_struct = quote! {
        #(#use_items)*
        #[clappen::__clappen_struct(prefix = $prefix, #default_prefix_arg)]
        #struct_def
        #(#prefixed_item_impls)*
    };

    // the nested call carries no prefix when nothing prefixes the field, so every key it forwards
    // is optional. A separate body, because `macro_rules!` forbids a once-bound metavariable
    // inside a repetition.
    let nested_item_impls: Vec<_> = regular_impls
        .iter()
        .map(|e| {
            quote! {
                #[clappen::__clappen_impl($(prefix = $prefix,)? prefixed_fields = [#(#fields),*], #default_prefix_arg)]
                #e
            }
        })
        .collect();

    let nested_struct = quote! {
        #(#use_items)*
        #[clappen::__clappen_struct($(prefix = $prefix,)? #default_prefix_arg)]
        #struct_def
        #(#nested_item_impls)*
    };

    // documents the generated macro itself (not the `#[clappen]` attribute)
    let macro_doc = " Invoke with `()` for the base struct, or `(\"prefix\")` for a prefixed copy. The `@__`-prefixed forms are internal and not part of the public API.";

    let prefixed_self_apply = &prefixed.self_apply.impls;
    let prefixed_child_apply = &prefixed.child_apply.invocations;
    let chained_self_apply = &chained.self_apply.impls;
    let chained_child_apply = &chained.child_apply.invocations;

    quote! {
        #[doc = #macro_doc]
        #[macro_export]
        macro_rules! #export_macro {
            // base instantiation: the plain struct. A template-free parent also emits each flattened
            // child's conversion here (`child_apply` with an empty prefix).
            () => {
                #default
                #(#base_child_apply)*
            };
            // prefixed instantiation: the prefixed struct, then this struct's own conversion
            // (`self_apply`) and a call into each flattened child's macro (`child_apply`). Emitted
            // inline rather than via a `@__template` self-call, which can't resolve across crates.
            ($prefix: literal) => {
                #prefixed_struct
                #(#prefixed_self_apply)*
                #(#prefixed_child_apply)*
            };

            // internal arms, not part of the public API:
            // struct only, no impls: builds a nested field's type for `clappen_command`
            (@__struct $($prefix: literal)?) => {
                #nested_struct
            };
            // reached when this struct is flattened in a parent (via a parent's `child_apply`): the
            // same `self_apply` + `child_apply` as the prefixed arm, at the chain the parent passed
            (@__template $($prefix: literal,)? chain = [
                $( ( $($command_prefix: literal,)? $field: ident $(, $parent_default: literal)? ) ),* $(,)?
            ]) => {
                #(#chained_self_apply)*
                #(#chained_child_apply)*
            };
        }
    }
}
