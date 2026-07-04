//! A template `From<&Prefixed>` (by-reference tag param): the leading reference is peeled, so the
//! borrowed value's field reads are prefixed the same way a by-value param's are.

#[clappen::clappen(export = prefixed_struct_generator)]
mod prefixed_struct_generator {
    #[derive(Debug, PartialEq)]
    struct ServerOptions {
        url: String,
    }

    // `Prefixed` here is a `&` param, so `value.url` reads through a reference
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
