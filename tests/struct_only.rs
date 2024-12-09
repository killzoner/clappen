// cargo expand --test struct_only
#[allow(dead_code)]
#[cfg(test)]
mod tests {
    #[clappen::__clappen_struct(prefix = "nested")]
    pub struct MyStruct {}

    #[clappen::__clappen_struct(prefix = "test")]
    pub struct ServerOptions {
        /// Address to connect to.
        ///
        address: String,

        /// Do you need to say hello?.
        ///
        config: Option<bool>,

        /// A nested struct that needs a prefix.
        ///
        nested: NestedMyStruct,
    }

    #[clappen::__clappen_impl(prefix = "test", prefixed_fields = [address, config])]
    impl ServerOptions {
        /// A function.
        ///
        fn a_function(&self) {
            println!("url: {}", self.address);
            println!("say_hello: {:?}", self.config);
        }
    }

    #[test]
    fn it_works() {
        let b = TestServerOptions {
            test_address: "url b".into(),
            test_config: Some(true),
            test_nested: NestedMyStruct {},
        };

        b.a_function();
    }
}
