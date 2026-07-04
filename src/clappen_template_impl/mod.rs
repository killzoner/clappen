pub(crate) mod attrs;
mod resolve;
mod rewrite;
pub(crate) mod template_impl;

// Constant
// the marker on a template impl; the clappen macro strips it
pub(crate) const IMPL_ATTR_CLAPPEN_TEMPLATE: &str = "clappen_template_impl";

// the tag keys, on the marker and on the forwarded attribute
const BASE_TAG_ATTR: &str = "base_tag";
const PREFIXED_TAG_ATTR: &str = "prefixed_tag";

// the other keys of the forwarded attribute
const STRUCT_IDENT_ATTR: &str = "struct_ident";
const CHAIN_ATTR: &str = "chain";

// the default tag idents
const DEFAULT_BASE_TAG: &str = "Base";
const DEFAULT_PREFIXED_TAG: &str = "Prefixed";
