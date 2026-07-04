//! A struct literal of the prefixed type, with explicit and shorthand fields: a renamed shorthand
//! field must become `renamed: original`.

#[clappen::clappen(export = prefixed_struct_generator)]
mod prefixed_struct_generator {
    #[derive(Debug, PartialEq)]
    struct ServerOptions {
        url: String,
        say_hello: Option<bool>,
    }

    // `Self` is the prefixed type here
    #[clappen_template_impl]
    impl From<Base> for Prefixed {
        fn from(value: Base) -> Self {
            let say_hello = value.say_hello;
            Self {
                url: value.url, // explicit init
                say_hello,      // shorthand init
            }
        }
    }
}

prefixed_struct_generator!();
prefixed_struct_generator!("test1");

#[test]
fn prefixed_literal_explicit_and_shorthand() {
    let base = ServerOptions {
        url: String::from("a"),
        say_hello: Some(true),
    };
    let prefixed: Test1ServerOptions = base.into();
    assert_eq!(
        prefixed,
        Test1ServerOptions {
            test1_url: String::from("a"),
            test1_say_hello: Some(true),
        }
    );
}
