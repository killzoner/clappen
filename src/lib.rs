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
mod clappen_struct;
mod helper;

use proc_macro::TokenStream;
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

/// Generates the macro defining prefixed struct.
///
/// - content should start with a `mod` definition (which is not used in generated code, so put whatever you want)
///     - `export` argument is required and defines the name of the exported macro
///     - `default_prefix` argument is optional: adds prefix to all fields if specified
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
