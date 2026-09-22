fn main(){
    pub struct ServerOptions {
        /// Address to connect to.
        ///
        address: String,
    }

    #[clappen::__clappen_impl(prefix = "", prefixed_fields = [address])]
    impl ServerOptions {
        /// A function.
        ///
        fn a_function(&self) -> String {
            format!("url: {}", self.address)
        }
    }
}
