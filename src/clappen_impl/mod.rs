use syn::Ident;

use crate::helper::prefix::{DefaultPrefix, StructPrefix};

pub(crate) mod attrs;
pub(crate) mod item_impl;

/// Process an Item (a struct, enum, etc) and return a TokenStream
pub(crate) trait ProcessItem {
    fn process(
        &mut self,
        default_prefix: DefaultPrefix,
        prefix: StructPrefix,
        prefixed_fields: Vec<Ident>,
    ) -> syn::Result<proc_macro2::TokenStream>;
}
