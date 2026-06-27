// marker attribute users place on a template impl; the clappen macro detects and strips it
pub(crate) const TEMPLATE_MARKER: &str = "clappen_template_impl";

// default template tag idents; overridable via `#[clappen_template_impl(base = .., prefixed = ..)]`
pub(crate) const DEFAULT_BASE_TAG: &str = "Base";
pub(crate) const DEFAULT_PREFIXED_TAG: &str = "Prefixed";

pub(crate) mod attrs;
pub(crate) mod expansion;
pub(crate) mod template;
