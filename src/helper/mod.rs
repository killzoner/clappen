use proc_macro2::{Ident, Span};
use quote::format_ident;
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::{LitStr, Result, Token, meta::ParseNestedMeta};

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

// a `name = [a, b]` attribute value
pub(crate) fn parse_bracketed<T: Parse>(meta: &ParseNestedMeta) -> Result<Vec<T>> {
    let value = meta.value()?;
    let content;
    syn::bracketed!(content in value);

    Ok(Punctuated::<T, Token![,]>::parse_terminated(&content)?
        .into_iter()
        .collect())
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
    let name = match prefix {
        Some(prefix) => format!("{}_{}", snake_case(prefix), snake_case(base)),
        None => snake_case(base),
    };

    Ident::new(&camel_case(&name), Span::call_site())
}

// module wrapping a nested field's struct, e.g. __inner_my_field
pub(crate) fn macro_module_name(field_ident: &str) -> Ident {
    format_ident!("__inner_{}", snake_case(field_ident))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_and_child_name_the_same_type() {
        let cases = [
            (None, None, "Remote"),
            (Some("d"), None, "Remote"),
            (Some("d"), Some("test"), "Remote"),
            (Some("my_d"), Some("test"), "Remote"),
            (Some("log"), Some("test"), "HTTPServer"),
        ];

        for (child_default, command_prefix, struct_ident) in cases {
            let child_default = child_default.map(str::to_string);
            let command_prefix = command_prefix.map(str::to_string);

            // what `child!()` names the struct: `DRemote`
            let base_name = prefixed_ident(&field_prefix(&child_default, &None), struct_ident);
            let nested = nested_step_prefix(&command_prefix, &None, &None);

            let parent_reference = prefixed_ident(&nested, &base_name.to_string());
            let child_definition =
                prefixed_ident(&field_prefix(&child_default, &nested), struct_ident);

            assert_eq!(
                parent_reference, child_definition,
                "default_prefix {child_default:?}, prefix {command_prefix:?}, {struct_ident}"
            );
        }
    }
}
