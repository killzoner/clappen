#![allow(dead_code)]

// A user type named `Base` in scope would clash with the default tag; `base = ..`/`prefixed = ..`
// pick custom tag idents that avoid it.
struct Base {
    other: u64,
}

#[clappen::clappen(export = options)]
mod options {
    struct Options {
        url: String,
    }

    #[clappen_template_impl(base = Canonical, prefixed = Variant)]
    impl From<Variant> for Canonical {
        fn from(value: Variant) -> Self {
            Self { url: value.url }
        }
    }
}

options!();
options!("test"); // snapshot: `impl From<TestOptions> for Options`

fn main() {}
