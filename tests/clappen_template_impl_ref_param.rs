//! A `&Prefixed` param: its field reads are prefixed like a by-value param's.

#[clappen::clappen(export = prefixed_struct_generator)]
mod prefixed_struct_generator {
    #[derive(Debug, PartialEq)]
    struct ServerOptions {
        url: String,
    }

    #[clappen_template_impl]
    impl From<&Prefixed> for Base {
        fn from(value: &Prefixed) -> Self {
            Self {
                url: value.url.clone(),
            }
        }
    }
}

prefixed_struct_generator!();
prefixed_struct_generator!("test1");

#[test]
fn from_reference_prefixes_borrowed_fields() {
    let prefixed = Test1ServerOptions {
        test1_url: String::from("a"),
    };
    assert_eq!(
        ServerOptions::from(&prefixed),
        ServerOptions {
            url: String::from("a"),
        }
    );
}
