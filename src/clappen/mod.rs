use attrs::Attributes;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Item, Result};

use crate::helper::PrefixValue;

pub(crate) mod attrs;

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

    let arm_body = |attribute_prefix: TokenStream| {
        let default_prefix = attrs.default_prefix.value().as_slice();
        let item_impls = items_impl.iter().map(|e| {
            quote! {
                #[clappen::__clappen_impl(#attribute_prefix #(default_prefix = #default_prefix,)* prefixed_fields = [#(#fields),*])]
                #e
            }
        });

        quote! {
            #(#use_items)*
            #[clappen::__clappen_struct(#attribute_prefix #(default_prefix = #default_prefix,)*)]
            #struct_def
            #(#item_impls)*
        }
    };
    let base_struct = arm_body(TokenStream::new());
    let prefixed_struct = arm_body(quote! { prefix = $prefix, });

    let macro_doc = " Invoke with `()` for the base struct, or `(\"prefix\")` for a prefixed copy.";

    Ok(quote! {
        #[doc = #macro_doc]
        #[macro_export]
        macro_rules! #export_macro {
            () => {
                #base_struct
            };
            ($prefix: literal) => {
                #prefixed_struct
            };
        }
    })
}
