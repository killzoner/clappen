pub(crate) mod attrs;
pub(crate) mod item_struct;

// Constant
const FIELD_ATTR_CLAPPEN_COMMAND: &str = "clappen_command";
const FIELD_ATTR_CLAPPEN_COMMAND_APPLY: &str = "apply";

const FIELD_ATTR_CLAP_FLATTEN_COMMAND: &str = "command";
const FIELD_ATTR_CLAP_FLATTEN_COMMAND_FLATTEN: &str = "flatten";

/// Process an Item (a struct, enum, etc) and return a TokenStream
pub(crate) trait ProcessItem {
    fn process(
        &mut self,
        default_prefix: String,
        prefix: String,
    ) -> syn::Result<proc_macro2::TokenStream>;
}
