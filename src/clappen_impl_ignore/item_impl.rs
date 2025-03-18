use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::ItemImpl;

use super::ProcessItem;

impl ProcessItem for ItemImpl {
    fn process(&mut self, ignore_self: Option<String>) -> syn::Result<TokenStream> {
        let item = &self;
        let self_ty = item.self_ty.to_token_stream().to_string().replace(" ", "");
        let mut ignore_impl = false;

        if let Some(tty) = ignore_self {
            ignore_impl = ignore_impl || self_ty.to_string().eq(&tty);
        }

        if ignore_impl {
            return Ok(quote! {});
        }

        Ok(self.to_token_stream())
    }
}
