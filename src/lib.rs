// local setup : rustup override set nightly
// undo : rustup override unset
// list overrides: rustup toolchain list
// #![feature(trace_macros)]
// trace_macros!(true);

#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::print_stderr)]
#![warn(clippy::print_stdout)]

mod clappen;
mod clappen_command;
mod clappen_impl;
mod clappen_struct;
mod helper;

use proc_macro::TokenStream;
use syn::{ItemImpl, ItemMod, ItemStruct, parse_macro_input};

#[doc(hidden)]
#[proc_macro_attribute]
pub fn __clappen_struct(args: TokenStream, target: TokenStream) -> TokenStream {
    // handle attributes
    let attrs = parse_macro_input!(args as clappen_struct::attrs::Attributes);

    // handle fields
    let item = parse_macro_input!(target as ItemStruct);

    clappen_struct::item_struct::expand(item, attrs)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[doc(hidden)]
#[proc_macro_attribute]
pub fn __clappen_impl(args: TokenStream, target: TokenStream) -> TokenStream {
    // handle attributes
    let attrs = parse_macro_input!(args as clappen_impl::attrs::Attributes);

    // handle fields
    let item = parse_macro_input!(target as ItemImpl);

    clappen_impl::item_impl::expand(item, attrs)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Generates the macro defining prefixed struct.
///
/// - content should start with a `mod` definition (which is not used in generated code, so put whatever you want)
///     - `export` argument is required and defines the name of the exported macro
///     - `default_prefix` argument is optional: adds prefix to all fields if specified
///
/// - invoke the generated macro: `NAME!()` for the base struct, `NAME!("prefix")` for the prefixed struct
///
/// - fields can use `#[clappen_command(apply = <my_exported_macro_name>)]` with `flatten` from `clap`
///   to reference already exported macros and generate a prefix
///     - `apply` is mandatory
///     - `prefix` is optional
///
/// Prefixes are preserved across multiple levels of nested structs.
#[proc_macro_attribute]
pub fn clappen(args: TokenStream, target: TokenStream) -> TokenStream {
    // handle mod definition
    let target2: proc_macro2::TokenStream = target.clone().into();
    let Some(content) = parse_macro_input!(target as ItemMod).content else {
        return syn::Error::new_spanned(target2, "clappen must be used on mod only")
            .into_compile_error()
            .into();
    };

    let items = content.1;

    // handle attributes
    let cloned_args = args.clone();
    let attrs = parse_macro_input!(cloned_args as clappen::attrs::Attributes);

    clappen::create_template(args.into(), attrs, items).into()
}
