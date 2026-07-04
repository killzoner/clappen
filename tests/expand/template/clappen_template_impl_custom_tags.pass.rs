#![allow(dead_code)]

// custom tags, because a user type named `Base` would clash with the default tag
struct Base {
    other: u64,
}

#[clappen::clappen(export = options)]
mod options {
    struct ServerOptions {
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
options!("test");

fn main() {}
