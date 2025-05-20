#[clappen::clappen(export = prefixed_struct_generator)]
mod m1 {
    pub struct ServerOptions {
        /// Address to connect to.
        ///
        url: String,

        /// Do you need to say hello?.
        ///
        say_hello: Option<bool>,
    }

    impl ServerOptions {
        fn a_function(&self) {
            println!("url: {}", self.url);
            println!("say_hello: {:?}", self.say_hello);
        }
        fn another_function(&self) {}
    }

    impl ServerOptions {
        fn a_third_function_in_second_impl_block(&self) {}
    }

    #[clappen::clappen_impl_custom(ignore_self = "ServerOptions")]
    #[allow(clippy::from_over_into)]
    impl Into<ServerOptions> for ServerOptions {
        fn into(self) -> ServerOptions {
            let Self { test_url, test_say_hello } = self;
            ServerOptions {
                url: test_url.into(),
                say_hello: test_say_hello.into(),
            }
        }
    }
}

fn main() {
    prefixed_struct_generator!();
    prefixed_struct_generator!("test");
}
