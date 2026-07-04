use proc_macro2::{Ident, Span};
use quote::format_ident;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{LitStr, Result, Token, meta::ParseNestedMeta};

pub(crate) mod prefix;

// the prefix attribute keys, used by the parsers and by the error messages
pub(crate) const PREFIX_ATTR: &str = "prefix";
pub(crate) const DEFAULT_PREFIX_ATTR: &str = "default_prefix";

// the value of a prefix, which is None when the prefix is empty
pub(crate) trait PrefixValue {
    fn value(&self) -> &Option<String>;

    // the prefix as a string, empty when it has no value
    fn as_str(&self) -> &str {
        self.value().as_deref().unwrap_or_default()
    }
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

// joins the parts in snake case, skipping None
fn snake_join(parts: &[&Option<String>]) -> String {
    let mut acc = String::new();

    for part in parts.iter().copied().flatten() {
        if acc.is_empty() {
            acc = part.to_string();
        } else {
            acc = snake_case(&format!("{part}_{acc}"));
        }
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

// the value of a `name = ".."` attribute, which must not be empty
fn parse_non_empty_literal(input: ParseStream, name: &str) -> Result<String> {
    let literal: LitStr = input.parse()?;
    let value = literal.value();

    if value.is_empty() {
        return Err(syn::Error::new(
            literal.span(),
            format!("'{name}' must not be empty"),
        ));
    }

    Ok(value)
}

fn non_empty(value: String) -> Option<String> {
    if value.is_empty() {
        return None;
    }

    Some(value)
}

// type ident: prefix prepended to a base name, in camel case
fn prefixed_ident(prefix: &Option<String>, base: &str) -> Ident {
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

// a value parsed from a string literal, so a test can build one without a token stream
#[cfg(test)]
pub(crate) fn parse_literal<P: Parse>(value: &str) -> P {
    let literal = LitStr::new(value, Span::call_site());

    syn::parse_quote!(#literal)
}
