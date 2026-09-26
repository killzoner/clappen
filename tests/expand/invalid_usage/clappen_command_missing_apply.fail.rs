
fn main(){
    #[clappen::__clappen_struct(prefix = "nested")]
    #[derive(clap::Args)]
    pub struct MyStruct {}

    #[clappen::__clappen_struct(prefix = "test")]
    #[derive(clap::Args)]
    pub struct ServerOptions {
        /// Address to connect to.
        ///
        address: String,

        /// A nested struct that needs a prefix.
        ///
        #[command(flatten)]
        #[clappen_command(prefix = "db")]
        nested: NestedMyStruct,
    }
}
