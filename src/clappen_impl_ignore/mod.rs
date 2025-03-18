pub(crate) mod attrs;
pub(crate) mod item_impl;

/// Process an Item (a struct, enum, etc) and return a TokenStream
pub(crate) trait ProcessItem {
    fn process(&mut self, ignore_self: Option<String>) -> syn::Result<proc_macro2::TokenStream>;
}
