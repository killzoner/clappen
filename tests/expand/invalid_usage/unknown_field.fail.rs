
fn main(){
    #[clappen::__clappen_struct(prefix = "test")]
    pub struct ServerOptions {
        /// Address to connect to.
        ///
        address: String,

        /// Do you need to say hello?.
        ///
        config: Option<bool>,
    }

    #[clappen::__clappen_impl(prefix = "test", prefixed_fields = [unknown, config])]
    impl ServerOptions {
        /// A function.
        ///
        fn a_function(&self) -> String {
            format!("url: {}, say_hello: {:?}", self.address, self.config)
        }
    }
}
