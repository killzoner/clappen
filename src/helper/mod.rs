use proc_macro2::{Ident, Span};
use quote::format_ident;
use syn::{LitStr, Result};

/// Prefix adds each prefix in order, skipping the ones that are not set.
/// result is constructed using snake_case
///
fn prefix(prefixes: &[&Option<String>]) -> String {
    let mut e = String::new();

    for prefix in prefixes.iter().copied().flatten() {
        if e.is_empty() {
            e = prefix.to_string();
        } else {
            e = snake_case(&format!("{prefix}_{e}"));
        }
    }

    e
}

// taken straight from paste crate (https://github.com/dtolnay/paste/blob/6a302522990cbfd9de4e0c61d91854622f7b2999/src/segment.rs#L176)
fn snake_case(elt: &str) -> String {
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
fn camel_case(elt: &str) -> String {
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

// a prefix attribute value, which must not be empty
pub(crate) fn require_non_empty(attribute: LitStr, name: &Ident) -> Result<String> {
    let value = attribute.value();

    if value.is_empty() {
        return Err(syn::Error::new(
            attribute.span(),
            format!("'{name}' must not be empty"),
        ));
    }

    Ok(value)
}

pub(crate) fn non_empty(prefix: String) -> Option<String> {
    if prefix.is_empty() {
        return None;
    }

    Some(prefix)
}

// prefix for a struct's field names
pub(crate) fn field_prefix(
    default_prefix: &Option<String>,
    struct_prefix: &Option<String>,
) -> Option<String> {
    non_empty(snake_case(&prefix(&[default_prefix, struct_prefix])))
}

// struct field name with its prefix: `<prefix>_<name>`
pub(crate) fn prefixed_field(prefix: &Option<String>, name: &str) -> String {
    let Some(prefix) = prefix else {
        return name.to_string();
    };

    format!("{prefix}_{name}")
}

// prefix literal passed one level down to a nested field's generated macro
pub(crate) fn nested_step_prefix(
    command_prefix: &Option<String>,
    default_prefix: &Option<String>,
    struct_prefix: &Option<String>,
) -> Option<String> {
    non_empty(camel_case(&prefix(&[
        command_prefix,
        default_prefix,
        struct_prefix,
    ])))
}

// type/struct ident: prefix prepended (camelCased) to a base name
pub(crate) fn prefixed_ident(prefix: &Option<String>, base: &str) -> Ident {
    let prefix = prefix.as_deref().map(snake_case).unwrap_or_default();

    Ident::new(&camel_case(&format!("{prefix}{base}")), Span::call_site())
}

// module wrapping a nested field's struct, e.g. __inner_my_field
pub(crate) fn macro_module_name(field_ident: &str) -> Ident {
    format_ident!("__inner_{}", snake_case(field_ident))
}
