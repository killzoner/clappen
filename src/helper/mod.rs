use proc_macro2::{Ident, Span};
use quote::format_ident;

/// Prefix adds each prefix in order, checking for empty prefix.
/// result is constructed using snake_case
///
fn prefix(prefixes: &[&str]) -> String {
    let mut e: String = "".into();

    for prefix in prefixes {
        if e.is_empty() {
            e = prefix.to_string();
        } else if !prefix.is_empty() {
            e = snake_case(format!("{prefix}_{e}"));
        }
    }

    e
}

// taken straight from paste crate (https://github.com/dtolnay/paste/blob/6a302522990cbfd9de4e0c61d91854622f7b2999/src/segment.rs#L176)
fn snake_case(elt: String) -> String {
    let mut acc = String::new();
    let mut prev = '_';
    for ch in elt.chars() {
        if ch.is_uppercase() && prev != '_' {
            acc.push('_');
        }
        acc.push(ch);
        prev = ch;
    }

    acc.to_lowercase() // only modification
}

// taken straight from paste crate (https://github.com/dtolnay/paste/blob/6a302522990cbfd9de4e0c61d91854622f7b2999/src/segment.rs#L176)
fn camel_case(elt: String) -> String {
    let mut acc = String::new();
    let mut prev = '_';
    for ch in elt.chars() {
        if ch != '_' {
            if prev == '_' {
                for chu in ch.to_uppercase() {
                    acc.push(chu);
                }
            } else if prev.is_uppercase() {
                for chl in ch.to_lowercase() {
                    acc.push(chl);
                }
            } else {
                acc.push(ch);
            }
        }
        prev = ch;
    }

    acc
}

// snake_case prefix for a struct's fields at (default_prefix, struct_prefix)
pub(crate) fn field_prefix(default_prefix: &str, struct_prefix: &str) -> String {
    snake_case(prefix(&[default_prefix, struct_prefix]))
}

// prefix literal passed one level down to a nested field's generated macro
pub(crate) fn nested_step_prefix(
    command_prefix: &str,
    default_prefix: &str,
    struct_prefix: &str,
) -> String {
    camel_case(prefix(&[command_prefix, default_prefix, struct_prefix]))
}

// type/struct ident: prefix prepended (camelCased) to a base name
pub(crate) fn prefixed_ident(prefix: &str, base: &str) -> Ident {
    Ident::new(&camel_case(format!("{prefix}{base}")), Span::call_site())
}

// module wrapping a nested field's struct, e.g. __inner_my_field
pub(crate) fn macro_module_name(field_ident: &str) -> Ident {
    format_ident!("__inner_{}", snake_case(field_ident.to_string()))
}
