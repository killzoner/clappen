fn main() {
    #[allow(non_snake_case)]
    mod __inner_MyStruct {
        /// Macros used for nested struct definition : []
        /// Struct with prefix '', default_prefix: ''
        pub struct MyStruct {}
    }
    /// Macros used for nested struct definition : [nested]
    pub struct ServerOptions {
        /// Address to connect to.
        ///
        url: String,
        /// Do you need to say hello?.
        ///
        say_hello: Option<bool>,
        /// A nested struct that needs a prefix.
        ///
        nested: __inner_MyStruct::MyStruct,
    }
}
