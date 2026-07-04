#[clappen::clappen(export = options)]
mod options {
    struct Options {
        url: String,
    }

    // `bad_tag` is not a recognized marker key; only `base_tag`/`prefixed_tag` are allowed.
    #[clappen_template_impl(bad_tag = Canonical)]
    impl From<Prefixed> for Base {
        fn from(value: Prefixed) -> Self {
            Self { url: value.url }
        }
    }
}

fn main() {}
