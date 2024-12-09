fn main() {
    /// Macros used for nested struct definition : []
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
            {
                ::std::io::_print(format_args!("url: {0}\n", self.url));
            };
            {
                ::std::io::_print(format_args!("say_hello: {0:?}\n", self.say_hello));
            };
        }
        fn another_function(&self) {}
    }
    impl ServerOptions {
        fn a_third_function_in_second_impl_block(&self) {}
    }
    /// Macros used for nested struct definition : []
    /// Struct with prefix 'test', default_prefix: ''
    pub struct TestServerOptions {
        /// Address to connect to.
        ///
        test_url: String,
        /// Do you need to say hello?.
        ///
        test_say_hello: Option<bool>,
    }
    /// Fields with prefix: [url,say_hello]
    impl TestServerOptions {
        fn a_function(&self) {
            {
                ::std::io::_print(format_args!("url: {0}\n", self.test_url));
            };
            {
                ::std::io::_print(
                    format_args!("say_hello: {0:?}\n", self.test_say_hello),
                );
            };
        }
        fn another_function(&self) {}
    }
    /// Fields with prefix: [url,say_hello]
    impl TestServerOptions {
        fn a_third_function_in_second_impl_block(&self) {}
    }
}
