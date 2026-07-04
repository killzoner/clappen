#![allow(dead_code)]

// A user type named `Base` in scope would clash with the default tag; `base_tag = ..`/`prefixed_tag = ..`
// pick custom tag idents that avoid it.
struct Base {
    other: u64,
}

#[clappen::clappen(export = options)]
mod options {
    struct Options {
        url: String,
    }

    #[clappen_template_impl(base_tag = Canonical, prefixed_tag = Variant)]
    impl From<Variant> for Canonical {
        fn from(value: Variant) -> Self {
            Self { url: value.url }
        }
    }
}

options!();
options!("test"); // snapshot: `impl From<TestOptions> for Options`

fn main() {}
