use attrs::Attributes;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Item;

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
                .to_compile_error()
        }
    };

    let items_impl: Vec<_> = items
        .iter()
        .flat_map(|e| match e {
            Item::Impl(item) => Some(item),
            _ => None,
        })
        .collect();

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

    let prefixed_item_impls: Vec<_> = items_impl
        .iter()
        .map(|e| {
            quote! {
                #[clappen::__clappen_impl(prefix = $prefix, prefixed_fields = [#(#fields)*], default_prefix = #default_prefix)]
                #e
            }
        })
        .collect();

    let fields_assignment: Vec<_> = struct_def.fields.iter().flat_map(|e| &e.ident).map(|i| {
      quote! {let #i = self.#i;}
    }).collect();
    let base_struct_name = struct_def.ident.clone();
    let span = base_struct_name.span();
    let base_struct_prefixed_name = default_prefix.to_owned() + &base_struct_name.to_string();
    let base_struct_prefixed = syn::Ident::new(&base_struct_prefixed_name, span);

    let into_impl = quote! {
      #[clappen::__clappen_impl(prefix = $prefix, prefixed_fields = [#(#fields)*], default_prefix = #default_prefix)]
      impl Into<#base_struct_prefixed> for #base_struct_prefixed{
        fn into(self) -> #base_struct_prefixed{
          #(#fields_assignment)*
          #base_struct_prefixed{
            #(#fields)*
          }
        }
      }
    };

    let default = match default_prefix {
        e if e.is_empty() => {
            quote! {
                #(#use_items)*
                #[clappen::__clappen_struct]
                #struct_def
                #(#items_impl)*
            }
        }
        _ => {
            let default_prefixed_item_impls: Vec<_> = items_impl
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

    quote! {
        #[macro_export]
        macro_rules! #export_macro {
            () => {
                #default
            };
            ($prefix: literal) => {
                #(#use_items)*
                #[doc=concat!(" Struct with prefix '", $prefix, "', default_prefix: '", #default_prefix, "'")]
                #[clappen::__clappen_struct(prefix = $prefix, default_prefix = #default_prefix)]
                #struct_def
                #(#prefixed_item_impls)*
                #into_impl
            };
        }
    }
}
