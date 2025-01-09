use proc_macro2::Span;
use quote::format_ident;
use syn::{spanned::Spanned, Expr, ExprArray, Ident, PathArguments};

/// Prefix adds each prefix in order, checking for empty prefix.
/// result is constructed using snake_case
///
pub(crate) fn prefix(prefixes: &[&str]) -> String {
    let mut res: String = "".into();

    for prefix in prefixes {
        if prefix.is_empty(){
            continue;
        }
        if res.is_empty() {
            res = prefix.to_string();
        } else if !prefix.is_empty() {
            res = snake_case(format!("{}_{}", prefix, res));
        }
    }

    res
}

// taken straight from paste crate (https://github.com/dtolnay/paste/blob/6a302522990cbfd9de4e0c61d91854622f7b2999/src/segment.rs#L176)
pub(crate) fn snake_case(elt: String) -> String {
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
pub(crate) fn camel_case(elt: String) -> String {
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

pub(crate) fn get_ident_from_path(path: &syn::Path) -> syn::Result<(syn::Ident, PathArguments)> {
    if let Some(seg) = path.segments.last() {
        let seg = seg.clone();
        Ok((seg.ident, seg.arguments))
    } else {
        Err(syn::Error::new(
            Span::call_site(),
            "Emtpy path for impl type",
        ))
    }
}

pub(crate) fn parse_prefixes(arr: ExprArray) -> syn::Result<Vec<String>> {
    fn flatten_match(e: &Expr) -> syn::Result<String> {
        match e {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(string),
                ..
            }) => {
                if string.parse::<proc_macro2::Ident>().is_ok() {
                    Ok(string.value())
                } else {
                    Err(syn::Error::new(
                        string.span(),
                        format!("prefixes must be valid idents, found {:?}", string),
                    ))
                }
            }
            syn::Expr::Path(syn::ExprPath { path: p, .. }) => {
                let (ident, _) = get_ident_from_path(p)?;
                Ok(ident.to_string())
            }
            syn::Expr::Group(syn::ExprGroup { expr: e, .. }) => flatten_match(e),
            _ => Err(syn::Error::new(
                e.span(),
                "prefixes must be literals or idents",
            )),
        }
    }

    arr.elems
        .iter()
        .map(flatten_match)
        .collect::<syn::Result<Vec<_>>>()
}

pub(crate) fn prefix_ident_str(
    ident: impl AsRef<str>,
    struct_prefix: &str,
    default_prefix: &str,
    is_field: bool,
) -> String {
    let mut ident = ident.as_ref().to_owned();
    let prefix = snake_case(prefix(&[default_prefix, struct_prefix]));

    // handle struct prefix
    if !prefix.is_empty() {
        if !is_field {
            // Struct style formatting
            ident.insert_str(0, &camel_case(prefix.clone()));
        } else if !ident.is_empty(){
            // Field style formatting
            ident.insert_str(0, format!("{}_", prefix).as_str());
        }else{
            ident.insert_str(0, &prefix);
        }
    }
    ident
}

pub(crate) fn prefix_struct_ident(ident: &mut Ident, struct_prefix: &str, default_prefix: &str) {
    let ident_str = ident.to_string();
    let prefixed = prefix_ident_str(ident_str, struct_prefix, default_prefix, false);
    *ident = Ident::new(&prefixed, ident.span());
}

pub(crate) fn prefix_field_ident(
    ident: &mut Option<Ident>,
    struct_prefix: &str,
    default_prefix: &str,
) -> syn::Result<()> {
    if let Some(field) = ident {
        let ident_str = field.to_string();
        let prefixed = prefix_ident_str(ident_str, struct_prefix, default_prefix, true);
        *field = Ident::new(&prefixed, field.span());
        Ok(())
    } else {
        Err(syn::Error::new(
            ident.span(),
            "Ident field could not be parsed",
        ))
    }
}

pub(crate) fn get_parents_from_prefixes(prefixes: &[String]) -> (String, Vec<String>) {
    let mut parents = prefixes.iter().rev().fold(Vec::new(), |mut vec, pre| {
        let prev = vec.last().cloned().unwrap_or_default();
        vec.push(prefix_ident_str(prev, pre, "", false));
        vec
    });
    parents.reverse();
    let name = if parents.is_empty() {
        Default::default()
    } else {
        parents.remove(0)
    };
    (name, parents)
}

pub(crate)  fn macro_module_name(mut field_type: Ident, nested_prefix: &str) -> Ident {
    prefix_struct_ident(&mut field_type, nested_prefix, "");
    let module_name = format_ident!("__inner_{}", field_type);

    module_name
}
