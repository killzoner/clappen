#[clappen::clappen(export = options)]
mod options {
    struct ServerOptions {
        url: String,
    }

    // the marker has no `= value` form
    #[clappen_template_impl = Foo]
    impl From<Prefixed> for Base {
        fn from(value: Prefixed) -> Self {
            Self { url: value.url }
        }
    }
}

fn main() {}
