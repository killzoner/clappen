#[clappen::clappen(export = nested1)]
mod nested {
    pub struct Nested1 {
        #[clappen_command(apply = nested2, prefix = "level2")]
        nested: Nested2
    }
}

#[clappen::clappen(export = nested2)]
mod nested {
    pub struct Nested2 {}
}

nested1!(); // define default nested struct

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
        #[clappen_command(apply = nested1, prefix = "level1")]
        nested: Nested1,
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
