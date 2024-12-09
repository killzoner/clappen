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
        #[clappen_command(apply = nested)]
        nested: MyStruct,
    }
}

fn main() {
    prefixed_struct_generator!();
}
