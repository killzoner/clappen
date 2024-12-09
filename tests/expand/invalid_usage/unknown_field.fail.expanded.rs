fn main() {
    /// Macros used for nested struct definition : []
    pub struct TestServerOptions {
        /// Address to connect to.
        ///
        test_address: String,
        /// Do you need to say hello?.
        ///
        test_config: Option<bool>,
    }
    /// Fields with prefix: [unknown,config]
    impl TestServerOptions {
        /// A function.
        ///
        fn a_function(&self) -> String {
            ::alloc::__export::must_use({
                let res = ::alloc::fmt::format(
                    format_args!(
                        "url: {0}, say_hello: {1:?}", self.address, self.test_config,
                    ),
                );
                res
            })
        }
    }
}
