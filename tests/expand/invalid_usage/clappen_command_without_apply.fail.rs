
fn main(){
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
        #[clappen_command(apply)]
        nested: NestedMyStruct,
    }
}

