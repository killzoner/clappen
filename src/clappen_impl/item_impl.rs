use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use std::str::FromStr;
use syn::{Ident, ItemImpl, Type, parse_quote};

use crate::clappen_impl::attrs;
use crate::helper::{PrefixValue, prefix::FieldPrefix};

// Prefixes the impl type and the `self.field` reads inside its methods.
pub(crate) fn expand(mut item: ItemImpl, attrs: attrs::Attributes) -> syn::Result<TokenStream> {
    let attrs::Attributes {
        prefix: attrs_prefix,
        prefixed_fields,
        default_prefix,
    } = attrs;

    let field_prefix = FieldPrefix::new(&default_prefix, &attrs_prefix);

    // handle impl ty prefix
    if field_prefix.value().is_some()
        && let Type::Path(path) = item.self_ty.as_mut()
        && let Some(segment) = path.path.segments.last_mut()
    {
        segment.ident = field_prefix.type_ident(&segment.ident.to_string());
    }

    // handle renaming of self fields references
    if field_prefix.value().is_some() {
        for i in item.items.iter_mut() {
            for field in &prefixed_fields {
                let content = i.to_token_stream().to_string();

                let field = field.to_string();
                let origin = format!("self.{field}");
                let replace = format!("self.{}", field_prefix.field_name(&field));
                let content = content.replace(&origin, &replace);

                let token = TokenStream::from_str(content.as_str())?;
                *i = parse_quote! {#token};
            }
        }
    }

    let doc_prefixed_fields = prefixed_fields
        .iter()
        .map(Ident::to_string)
        .collect::<Vec<_>>()
        .join(",");
    // re-emit whole impl to keep trait and generics
    let item = &item;

    Ok(quote! {
        #[doc=concat!(concat!(" Fields with prefix: [", #doc_prefixed_fields, "]"))]
        #item
    })
}
