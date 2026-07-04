#[clappen::clappen(export = options)]
mod options {
    struct ServerOptions {
        url: String,
    }

    // only `base_tag` and `prefixed_tag` are marker keys
    #[clappen_template_impl(bad_tag = Canonical)]
    impl From<Prefixed> for Base {
        fn from(value: Prefixed) -> Self {
            Self { url: value.url }
        }
    }
}

fn main() {}
