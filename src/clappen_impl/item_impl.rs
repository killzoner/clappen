use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use std::str::FromStr;
use syn::spanned::Spanned;
use syn::{parse_quote, Ident, ItemImpl};

use super::ProcessItem;
use crate::helper;

impl ProcessItem for ItemImpl {
    fn process(
        &mut self,
        default_prefix: String,
        attrs_prefix: String,
        prefixed_fields: Vec<String>,
    ) -> syn::Result<TokenStream> {
        let item = &self;
        let mut self_ty = item.self_ty.to_token_stream();

        let prefix = helper::snake_case(helper::prefix(&[
            default_prefix.as_str(),
            attrs_prefix.as_str(),
        ]));

        // Preserve surrounding impl tokens
        let outer_attrs: Vec<_> = self
            .attrs
            .iter()
            .filter(|a| matches!(a.style, syn::AttrStyle::Outer))
            .collect();

        // Handle generics and traits
        let (impl_generics, _, where_clause) = self.generics.split_for_impl();
        let trait_tokens = self
            .trait_
            .clone()
            .map(|(not, path, f)| quote! {#not #path #f});

        let Self {
            defaultness,
            unsafety,
            impl_token,
            ..
        } = &self;

        // validate usage
        let (ident, generics) = if let syn::Type::Path(p) = *item.self_ty.clone() {
            if p.path.segments.len() != 1 {
                return Err(syn::Error::new(p.span(), "path must only have length of 1"));
            }
            if let Some(seg) = p.path.segments.last() {
                let seg = seg.clone();
                (seg.ident, seg.arguments)
            } else {
                return Err(syn::Error::new(p.span(), "Emtpy path for impl type"));
            }
        } else {
            return Err(syn::Error::new(
                item.span(),
                "Impl type can only be a simple path",
            ));
        };

        let before_type = quote! {#(#outer_attrs)* #defaultness #unsafety #impl_token #impl_generics #trait_tokens};
        let after_type = quote! {#generics #where_clause};

        // handle impl ty prefix
        if !prefix.is_empty() {
            let mut ident_str = ident.to_string();
            ident_str.insert_str(0, &helper::camel_case(prefix.to_owned()));

            self_ty = Ident::new(&ident_str, ident.span()).to_token_stream();
        }

        // handle renaming of self fields references
        if !prefix.is_empty() {
            for i in self.items.iter_mut() {
                for field in &prefixed_fields {
                    let content = i.to_token_stream().to_string();

                    let origin = format!("self.{}", field);
                    let replace = format!("self.{}_{}", prefix, field);
                    let content = content.replace(&origin, &replace);

                    let token = TokenStream::from_str(content.as_str())?;
                    *i = parse_quote! {#token};
                }
            }
        }

        let doc_prefixed_fields = prefixed_fields.join(",");
        let items = &self.items;

        Ok(quote! {
            #[doc=concat!(concat!(" Fields with prefix: [", #doc_prefixed_fields, "]"))]
            #before_type #self_ty #after_type{
                #(#items)*
            }
        })
    }
}
