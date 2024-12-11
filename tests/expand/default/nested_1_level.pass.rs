#[clappen::clappen(export = nested)]
mod nested {
    pub struct MyStruct {}
}

#[clappen::clappen(export = prefixed_struct_generator)]
mod m1 {
    pub struct ServerOptions {
        /// Address to connect to.
        ///
        url: String,

        /// Do you need to say hello?.
        ///
        say_hello: Option<bool>,

        /// A nested struct that needs a prefix.
        ///
        #[clappen_command(apply = nested, prefix = "test")]
        nested: MyStruct,

        /// A nested struct that needs another prefix.
        ///
        #[clappen_command(apply = nested, prefix = "test1")]
        nested1: MyStruct,
    }

    impl ServerOptions {
        fn a_function(&self) -> String {
            format!("url: {}, say_hello: {:?}", self.url, self.say_hello)
        }
        fn another_function(&self) {}
    }

    impl ServerOptions {
        fn a_third_function_in_second_impl_block(&self) {}
    }
}

fn main() {
    prefixed_struct_generator!();
    prefixed_struct_generator!("test");
}
