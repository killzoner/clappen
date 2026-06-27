#[clappen::clappen(export = prefixed_struct_generator)]
mod prefixed_struct_generator {
    struct ServerOptions {
        url: String,
        say_hello: Option<bool>,
    }

    // same feature, `Into` direction
    #[clappen_template_impl]
    #[allow(clippy::from_over_into)]
    impl Into<Base> for Prefixed {
        fn into(self) -> Base {
            Base {
                url: self.url,
                say_hello: self.say_hello,
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
