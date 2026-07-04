#[clappen::clappen(export = prefixed_struct_generator)]
mod prefixed_struct_generator {
    struct ServerOptions {
        url: String,
        say_hello: Option<bool>,
    }

    // `Base` is the default struct, `Prefixed` the generated variant
    #[clappen_template_impl]
    impl From<Prefixed> for Base {
        fn from(value: Prefixed) -> Self {
            Self {
                url: value.url,
                say_hello: value.say_hello,
            }
        }
    }
}

prefixed_struct_generator!();
prefixed_struct_generator!("test");

fn main() {
    let test = TestServerOptions {
        test_url: String::from("Hello"),
        test_say_hello: Some(true),
    };
    let _: ServerOptions = test.into();
}
