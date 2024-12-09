pub(crate) mod attrs;
pub(crate) mod item_impl;

/// Process an Item (a struct, enum, etc) and return a TokenStream
pub(crate) trait ProcessItem {
    fn process(
        &mut self,
        default_prefix: String,
        prefix: String,
        prefixed_fields: Vec<String>,
    ) -> syn::Result<proc_macro2::TokenStream>;
}
