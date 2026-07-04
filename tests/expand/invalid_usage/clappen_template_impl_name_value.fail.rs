#[clappen::clappen(export = options)]
mod options {
    struct Options {
        url: String,
    }

    // the marker takes no `= value` form; use `#[clappen_template_impl]` or `(base_tag = .., prefixed_tag = ..)`.
    #[clappen_template_impl = Foo]
    impl From<Prefixed> for Base {
        fn from(value: Prefixed) -> Self {
            Self { url: value.url }
        }
    }
}

fn main() {}
