mod nested {
    /// Macros used for nested struct definition : []
    pub struct MyStruct {}
}
fn main() {
    use nested::MyStruct;
    /// Macros used for nested struct definition : []
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
