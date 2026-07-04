struct Fallback {
    url: String,
}

#[clappen::clappen(export = prefixed_struct_generator)]
mod prefixed_struct_generator {
    struct ServerOptions {
        url: String,
    }

    // the local reuses the parameter name, so `value` is a `Fallback` from that point on and its
    // field keeps its own name
    #[clappen_template_impl]
    impl From<Prefixed> for Base {
        fn from(value: Prefixed) -> Self {
            let url = value.url;
            let value = Fallback {
                url: String::from("fallback"),
            };

            Self {
                url: if url.is_empty() { value.url } else { url },
            }
        }
    }
}

prefixed_struct_generator!();
prefixed_struct_generator!("test");

fn main() {
    let test = TestServerOptions {
        test_url: String::from("Hello"),
    };
    let options: ServerOptions = test.into();

    assert_eq!(options.url, "Hello");
}
