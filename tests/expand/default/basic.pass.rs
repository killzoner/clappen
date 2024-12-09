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
}

fn main() {
    prefixed_struct_generator!();
    prefixed_struct_generator!("test");
}
