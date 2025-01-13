// local setup : rustup override set nightly
// undo : rustup override unset
// list overrides: rustup toolchain list
// #![feature(trace_macros)]
// trace_macros!(true);

#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::print_stderr)]
#![warn(clippy::print_stdout)]

mod clappen;
mod clappen_command;
mod clappen_impl;
mod clappen_into;
mod clappen_struct;
mod clappen_use;
mod helper;

use helper::{get_parents_from_prefixes, macro_module_name, prefix_ident_str};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemImpl, ItemMod, ItemStruct};

#[doc(hidden)]
#[proc_macro_attribute]
pub fn __clappen_struct(args: TokenStream, target: TokenStream) -> TokenStream {
    // handle attributes
    let cloned_args = args.clone();
    let mut attrs = clappen_struct::attrs::Attributes::default();
    let attrs_parser = syn::meta::parser(|meta| attrs.parse(meta));
    parse_macro_input!(cloned_args with attrs_parser);

    // handle fields
    let mut item = parse_macro_input!(target as ItemStruct);

    use clappen_struct::ProcessItem;
    let expanded = item.process(attrs.default_prefix, attrs.prefix);

    let expanded = match expanded {
        Ok(e) => e,
        Err(e) => e.to_compile_error(),
    };

    expanded.into()
}

#[doc(hidden)]
#[proc_macro_attribute]
pub fn __clappen_impl(args: TokenStream, target: TokenStream) -> TokenStream {
    // handle attributes
    let cloned_args = args.clone();
    let mut attrs = clappen_impl::attrs::Attributes::default();
    let attrs_parser = syn::meta::parser(|meta| attrs.parse(meta));
    parse_macro_input!(cloned_args with attrs_parser);

    // handle fields
    let mut item = parse_macro_input!(target as ItemImpl);

    use clappen_impl::ProcessItem;
    let expanded = item.process(attrs.default_prefix, attrs.prefix, attrs.prefixed_fields);

    let expanded = match expanded {
        Ok(e) => e,
        Err(e) => e.to_compile_error(),
    };

    expanded.into()
}

#[doc(hidden)]
#[proc_macro]
pub fn __clappen_use(input: TokenStream) -> TokenStream {
    let cloned_args = input.clone();
    let attrs = parse_macro_input!(cloned_args as clappen_use::Attributes);
    let (_, parents) = get_parents_from_prefixes(&attrs.prefixes);
    let len = parents.len();
    let uses = parents.iter().enumerate().map(|(i, s)| {
        let ident = syn::Ident::new(s, proc_macro2::Span::call_site());
        let module_name = if i + 1 == len {
            quote! {}
        } else {
            let name = macro_module_name(ident.clone(), "");
            quote! {
                #name::
            }
        };
        quote! {
            use super::#module_name #ident;
        }
    });
    quote! {#(#uses)*}.into()
}

#[doc(hidden)]
#[proc_macro]
pub fn __into_impl(input: TokenStream) -> TokenStream {
    let cloned_args = input.clone();
    let mut attrs = clappen_into::Attributes::default();
    let attrs_parser = syn::meta::parser(|meta| attrs.parse(meta));
    parse_macro_input!(cloned_args with attrs_parser);
    let prefix = attrs.prefixes.first().cloned().unwrap_or_default();
    let (item_str, parents) = get_parents_from_prefixes(&attrs.prefixes);
    // return debug_into(parents, attrs.prefixes, prefix);
    let item_ident = proc_macro2::Ident::new(&item_str, proc_macro2::Span::call_site());
    // Most readable iter transformation
    let mut field_prefixes =
        attrs
            .prefixes
            .iter()
            .rev()
            .skip(1)
            .fold(Vec::new(), |mut vec, pre| {
                let prev = vec.last().cloned().unwrap_or_default();
                vec.push(prefix_ident_str(prev, pre, "", true));
                vec
            });
    field_prefixes.reverse();
    field_prefixes.push(attrs.default_prefix);
    let len = field_prefixes.len();
    let intos = parents
        .clone()
        .into_iter()
        .zip(field_prefixes.windows(2))
        .map(|(p, sl)| {
            let &[pre_from, pre_into] = &sl else {
                unreachable!()
            };
            let prefix_fields = |pre: &str| -> Vec<_> {
                attrs
                    .fields
                    .iter()
                    .map(|f| prefix_ident_str(f, pre, "", true))
                    .map(|s| syn::Ident::new(&s, proc_macro2::Span::call_site()))
                    .collect()
            };
            let from_fields = prefix_fields(pre_from);
            let into_fields = from_fields.iter().zip(prefix_fields(pre_into)).map(|(f,i)|{
                quote! {
                    #i: #f.into()
                }
            });

            let ident = proc_macro2::Ident::new(&p, proc_macro2::Span::call_site());
            quote! {
                #[allow(clippy::from_over_into)]
                impl Into<#ident> for #item_ident{
                    fn into(self) -> #ident{
                        let Self{#(#from_fields),*} = self;
                        #ident{#(#into_fields),*}
                    }
                }
            }
        });
    let _debug = debug_into(parents, attrs.prefixes, prefix);
    quote! {
        #(#intos)*

    }
    .into()
}

fn debug_into(
    parents: Vec<String>,
    prefixes: Vec<String>,
    prefix: String,
) -> proc_macro2::TokenStream {
    let parent_len = parents.len();
    let prefix_len = prefixes.len();
    quote! {
        static parents: [& str; #parent_len] = [#(#parents),*];
        static prefixes: [& str; #prefix_len] = [#(#prefixes),*];
        static prefix: & str = #prefix;
    }
}

/// Generates the macro defining prefixed struct.
///
/// - content should start with a `mod` definition (which is not used in generated code, so put whatever you want)
///     - `export` argument is required and defines the name of the exported macro
///     - `default_prefix` argument is optional: adds prefix to all fields if specified
///     - `gen_into` argument is optional: makes generated structs impl Into<Base>, automatically generates base struct
///
/// - fields can use `#[clappen_command(apply = <my_exported_macro_name>]` with `flatten` from `clap`
///   to reference already exported macros and generate a prefix
///     - `apply` is mandatory
///     - `prefix` is optional
///    
/// Prefixes are preserved across multiple levels of nested structs.
#[proc_macro_attribute]
pub fn clappen(args: TokenStream, target: TokenStream) -> TokenStream {
    // handle attributes
    let cloned_args = args.clone();
    let mut attrs = clappen::attrs::Attributes::default();
    let attrs_parser = syn::meta::parser(|meta| attrs.parse(meta));
    parse_macro_input!(cloned_args with attrs_parser);

    // handle mod definition
    let target2: proc_macro2::TokenStream = target.clone().into();
    let items = parse_macro_input!(target as ItemMod).content.ok_or(
        syn::Error::new_spanned(target2, "clappen must be used on mod only").to_compile_error(),
    );

    let items = match items {
        Ok(e) => e.1,
        Err(e) => return TokenStream::from(e),
    };

    clappen::create_template(args.into(), attrs, items).into()
}
