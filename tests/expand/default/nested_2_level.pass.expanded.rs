fn main() {
    mod __inner_nested {
        mod __inner_level1_nested {
            /// Macros used for nested struct definition : []
            /// Struct with prefix 'Level1Level2', default_prefix: ''
            pub struct Level1Level2Nested2 {
                level1_level2_id: String,
            }
            /// Fields with prefix: [id]
            impl Level1Level2Nested2 {
                pub fn another_nested_function_2(&self) -> String {
                    self.level1_level2_id.clone()
                }
            }
        }
        /// Macros used for nested struct definition : [nested2]
        /// Struct with prefix 'Level1', default_prefix: ''
        pub struct Level1Nested1 {
            level1_nested: __inner_level1_nested::Level1Level2Nested2,
        }
        /// Fields with prefix: [nested]
        impl Level1Nested1 {
            pub fn another_nested_function_1(&self) -> String {
                self.level1_nested.another_nested_function_2()
            }
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
        pub fn another_nested_function(&self) -> String {
            self.nested.another_nested_function_1()
        }
    }
    impl ServerOptions {
        fn a_third_function_in_second_impl_block(&self) {}
    }
    mod __inner_test_nested {
        mod __inner_test_level1_nested {
            /// Macros used for nested struct definition : []
            /// Struct with prefix 'TestLevel1Level2', default_prefix: ''
            pub struct TestLevel1Level2Nested2 {
                test_level1_level2_id: String,
            }
            /// Fields with prefix: [id]
            impl TestLevel1Level2Nested2 {
                pub fn another_nested_function_2(&self) -> String {
                    self.test_level1_level2_id.clone()
                }
            }
        }
        /// Macros used for nested struct definition : [nested2]
        /// Struct with prefix 'TestLevel1', default_prefix: ''
        pub struct TestLevel1Nested1 {
            test_level1_nested: __inner_test_level1_nested::TestLevel1Level2Nested2,
        }
        /// Fields with prefix: [nested]
        impl TestLevel1Nested1 {
            pub fn another_nested_function_1(&self) -> String {
                self.test_level1_nested.another_nested_function_2()
            }
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
        pub fn another_nested_function(&self) -> String {
            self.test_nested.another_nested_function_1()
        }
    }
    /// Fields with prefix: [url,say_hello,nested]
    impl TestServerOptions {
        fn a_third_function_in_second_impl_block(&self) {}
    }
}
