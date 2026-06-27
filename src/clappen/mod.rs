use attrs::Attributes;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::Item;

use crate::clappen_template_impl::expansion::Expansion;

pub(crate) mod attrs;

pub(crate) fn create_template(
    args: TokenStream,
    attrs: Attributes,
    items: Vec<Item>,
) -> TokenStream {
    let export_name = attrs.export.ok_or(
        syn::Error::new_spanned(&args, "clappen 'export' attribute not found").to_compile_error(),
    );

    let export_macro = match export_name {
        Ok(e) => e,
        Err(e) => return e,
    };

    let default_prefix = &attrs.default_prefix;

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

    // struct field idents, comma-joined, forwarded to the prefixing macros
    let fields: Vec<_> = struct_def
        .fields
        .iter()
        .flat_map(|e| &e.ident)
        .enumerate()
        .map(|(index, e)| {
            // don't add a comma if it's last ident
            if index == struct_def.fields.len() - 1 {
                e.to_token_stream()
            } else {
                quote! {#e,}
            }
        })
        .collect();

    // split regular vs marked impls and build the per-prefix template arms
    let Expansion {
        regular_impls,
        prefixed_self_apply,
        prefixed_child_apply,
        chained_self_apply,
        chained_child_apply,
        base_child_apply,
    } = crate::clappen_template_impl::expansion::build(
        &items_impl,
        struct_def,
        &fields,
        default_prefix,
    );

    let prefixed_item_impls: Vec<_> = regular_impls
        .iter()
        .map(|e| {
            quote! {
                #[clappen::__clappen_impl(prefix = $prefix, prefixed_fields = [#(#fields)*], default_prefix = #default_prefix)]
                #e
            }
        })
        .collect();

    let default = match default_prefix {
        e if e.is_empty() => {
            quote! {
                #(#use_items)*
                #[clappen::__clappen_struct]
                #struct_def
                #(#regular_impls)*
            }
        }
        _ => {
            let default_prefixed_item_impls: Vec<_> = regular_impls
                .iter()
                .map(|e| {
                    quote! {
                        #[clappen::__clappen_impl(prefixed_fields = [#(#fields)*], default_prefix = #default_prefix)]
                        #e
                    }
                })
                .collect();

            quote! {
                #(#use_items)*
                #[doc=concat!(" Struct with prefix '', default_prefix: '", #default_prefix, "'")]
                #[clappen::__clappen_struct(default_prefix = #default_prefix)]
                #struct_def
                #(#default_prefixed_item_impls)*
            }
        }
    };

    // built once, used in both the `(@__struct $prefix)` and `($prefix)` arms
    let prefixed_struct = quote! {
        #(#use_items)*
        #[doc=concat!(" Struct with prefix '", $prefix, "', default_prefix: '", #default_prefix, "'")]
        #[clappen::__clappen_struct(prefix = $prefix, default_prefix = #default_prefix)]
        #struct_def
        #(#prefixed_item_impls)*
    };

    // documents the generated macro itself (not the `#[clappen]` attribute)
    let macro_doc = " Invoke with `()` for the base struct, or `(\"prefix\")` for a prefixed copy. The `@__`-prefixed forms are internal and not part of the public API.";

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
            (@__struct $prefix: literal) => {
                #prefixed_struct
            };
            // reached when this struct is flattened in a parent (via a parent's `child_apply`): the
            // same `self_apply` + `child_apply` as the prefixed arm, at the chain the parent passed
            (@__template $prefix: literal, chain = [ $( ($command_prefix: literal, $field: ident, $parent_default: literal) ),* $(,)? ]) => {
                #(#chained_self_apply)*
                #(#chained_child_apply)*
            };
        }
    }
}
