#[clappen::clappen(export = nested)]
mod nested {
    pub struct MyStruct {}
}

mod nested {
    nested!(); // define default nested struct in a mod
}

#[clappen::clappen(export = prefixed_struct_generator)]
mod m1 {
    use nested::MyStruct;
    pub struct ServerOptions {
        /// Address to connect to.
        ///
        url: String,

        /// Do you need to say hello?.
        ///
        say_hello: Option<bool>,

        /// A nested struct that needs a prefix.
        ///
        nested: MyStruct,
    }
}

fn main() {
    prefixed_struct_generator!();
}
