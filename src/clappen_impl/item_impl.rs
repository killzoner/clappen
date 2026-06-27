use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use std::str::FromStr;
use syn::{ItemImpl, Type, parse_quote};

use super::ProcessItem;
use crate::helper;

impl ProcessItem for ItemImpl {
    fn process(
        &mut self,
        default_prefix: String,
        attrs_prefix: String,
        prefixed_fields: Vec<String>,
    ) -> syn::Result<TokenStream> {
        let prefix = helper::field_prefix(&default_prefix, &attrs_prefix);

        // handle impl ty prefix
        if !prefix.is_empty()
            && let Type::Path(path) = self.self_ty.as_mut()
            && let Some(segment) = path.path.segments.last_mut()
        {
            segment.ident = helper::prefixed_ident(&prefix, &segment.ident.to_string());
        }

        // handle renaming of self fields references
        if !prefix.is_empty() {
            for i in self.items.iter_mut() {
                for field in &prefixed_fields {
                    let content = i.to_token_stream().to_string();

                    let origin = format!("self.{field}");
                    let replace = format!("self.{prefix}_{field}");
                    let content = content.replace(&origin, &replace);

                    let token = TokenStream::from_str(content.as_str())?;
                    *i = parse_quote! {#token};
                }
            }
        }

        let doc_prefixed_fields = prefixed_fields.join(",");
        // re-emit whole impl to keep trait and generics
        let item = &*self;

        Ok(quote! {
            #[doc=concat!(concat!(" Fields with prefix: [", #doc_prefixed_fields, "]"))]
            #item
        })
    }
}
