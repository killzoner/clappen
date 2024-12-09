mod __inner_nested {
    /// Macros used for nested struct definition : []
    /// Struct with prefix 'Level2', default_prefix: ''
    pub struct Level2Nested2 {}
}
/// Macros used for nested struct definition : [nested2]
pub struct Nested1 {
    nested: __inner_nested::Level2Nested2,
}
fn main() {
    mod __inner_nested {
        mod __inner_level1_nested {
            /// Macros used for nested struct definition : []
            /// Struct with prefix 'Level1Level2', default_prefix: ''
            pub struct Level1Level2Nested2 {}
        }
        /// Macros used for nested struct definition : [nested2]
        /// Struct with prefix 'Level1', default_prefix: ''
        pub struct Level1Nested1 {
            Level1_nested: __inner_level1_nested::Level1Level2Nested2,
        }
    }
    /// Macros used for nested struct definition : [nested1]
    pub struct ServerOptions {
        /// Address to connect to.
        ///
        url: String,
        /// Do you need to say hello?.
        ///
        say_hello: Option<bool>,
        /// A nested struct that needs a prefix.
        ///
        nested: __inner_nested::Level1Nested1,
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
    mod __inner_test_nested {
        mod __inner_test_level1_nested {
            /// Macros used for nested struct definition : []
            /// Struct with prefix 'TestLevel1Level2', default_prefix: ''
            pub struct TestLevel1Level2Nested2 {}
        }
        /// Macros used for nested struct definition : [nested2]
        /// Struct with prefix 'TestLevel1', default_prefix: ''
        pub struct TestLevel1Nested1 {
            TestLevel1_nested: __inner_test_level1_nested::TestLevel1Level2Nested2,
        }
    }
    /// Macros used for nested struct definition : [nested1]
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
        test_nested: __inner_test_nested::TestLevel1Nested1,
    }
    /// Fields with prefix: [url,say_hello,nested]
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
    /// Fields with prefix: [url,say_hello,nested]
    impl TestServerOptions {
        fn a_third_function_in_second_impl_block(&self) {}
    }
}
