// marker attribute users place on a template impl; the clappen macro detects and strips it
pub(crate) const IMPL_ATTR_CLAPPEN_TEMPLATE: &str = "clappen_template_impl";

// attribute keys naming the template tag idents; shared by the marker and the forwarded attr
const BASE_TAG_ATTR: &str = "base_tag";
const PREFIXED_TAG_ATTR: &str = "prefixed_tag";

// default template tag idents; overridable via `#[clappen_template_impl(base_tag = .., prefixed_tag = ..)]`
const DEFAULT_BASE_TAG: &str = "Base";
const DEFAULT_PREFIXED_TAG: &str = "Prefixed";

pub(crate) mod attrs;
mod resolve;
mod rewrite;
pub(crate) mod template_impl;
