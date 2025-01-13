fn main() {
    #[allow(non_snake_case)]
    mod __inner_TestMyStruct {
        /// Macros used for nested struct definition : []
        /// Struct with prefix 'Test', default_prefix: ''
        pub struct TestMyStruct {}
    }
    #[allow(non_snake_case)]
    mod __inner_Test1MyStruct {
        /// Macros used for nested struct definition : []
        /// Struct with prefix 'Test1', default_prefix: ''
        pub struct Test1MyStruct {}
    }
    /// Macros used for nested struct definition : [nested,nested]
    pub struct ServerOptions {
        /// Address to connect to.
        ///
        url: String,
        /// Do you need to say hello?.
        ///
        say_hello: Option<bool>,
        /// A nested struct that needs a prefix.
        ///
        nested: __inner_TestMyStruct::TestMyStruct,
        /// A nested struct that needs another prefix.
        ///
        nested1: __inner_Test1MyStruct::Test1MyStruct,
    }
    impl ServerOptions {
        fn a_function(&self) -> String {
            ::alloc::__export::must_use({
                let res = ::alloc::fmt::format(
                    format_args!("url: {0}, say_hello: {1:?}", self.url, self.say_hello),
                );
                res
            })
        }
        fn another_function(&self) {}
    }
    impl ServerOptions {
        fn a_third_function_in_second_impl_block(&self) {}
    }
    #[allow(non_snake_case)]
    mod __inner_TestTestMyStruct {
        /// Macros used for nested struct definition : []
        /// Struct with prefix 'TestTest', default_prefix: ''
        pub struct TestTestMyStruct {}
    }
    #[allow(non_snake_case)]
    mod __inner_TestTest1MyStruct {
        /// Macros used for nested struct definition : []
        /// Struct with prefix 'TestTest1', default_prefix: ''
        pub struct TestTest1MyStruct {}
    }
    /// Macros used for nested struct definition : [nested,nested]
    /// Struct with prefix 'test', default_prefix: ''
    pub struct TestServerOptions {
        /// Address to connect to.
        ///
        test_url: String,
        /// Do you need to say hello?.
        ///
        test_say_hello: Option<bool>,
        /// A nested struct that needs a prefix.
        ///
        test_nested: __inner_TestTestMyStruct::TestTestMyStruct,
        /// A nested struct that needs another prefix.
        ///
        test_nested1: __inner_TestTest1MyStruct::TestTest1MyStruct,
    }
    /// Fields with prefix: [url,say_hello,nested,nested1]
    impl TestServerOptions {
        fn a_function(&self) -> String {
            ::alloc::__export::must_use({
                let res = ::alloc::fmt::format(
                    format_args!(
                        "url: {0}, say_hello: {1:?}", self.test_url, self.test_say_hello,
                    ),
                );
                res
            })
        }
        fn another_function(&self) {}
    }
    /// Fields with prefix: [url,say_hello,nested,nested1]
    impl TestServerOptions {
        fn a_third_function_in_second_impl_block(&self) {}
    }
}
