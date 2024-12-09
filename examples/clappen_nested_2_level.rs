use clap::Parser;

#[clappen::clappen(export = nested1)]
mod nested {
    #[derive(clap::Args, Debug, Clone)]
    pub struct Nested1 {
        #[command(flatten)]
        #[clappen_command(apply = nested2, prefix = "level2")]
        nested: Nested2,
    }
}

#[clappen::clappen(export = nested2)]
mod nested {
    #[derive(clap::Args, Debug, Clone)]
    pub struct Nested2 {
        #[arg(env, long)]
        id: String,
    }
}

#[clappen::clappen(export = prefixed_struct_generator)]
mod m1 {
    /// A test command line
    #[derive(clap::Parser, Debug, Clone)]
    #[command(version)]
    pub struct ServerOptions {
        /// Address to connect to.
        ///
        #[arg(env, long)]
        url: String,

        /// Do you need to say hello?.
        ///
        #[arg(env, long)]
        say_hello: Option<bool>,

        /// A nested struct that needs a prefix.
        ///
        #[command(flatten)]
        #[clappen_command(apply = nested1, prefix = "level1")]
        nested: Nested1,
    }

    #[allow(dead_code)]
    impl ServerOptions {
        fn a_function(&self) -> String {
            format!("url: {}, say_hello: {:?}", self.url, self.say_hello)
        }
        fn another_function(&self) {}
    }

    #[allow(dead_code)]
    impl ServerOptions {
        fn a_third_function_in_second_impl_block(&self) {}
    }
}

prefixed_struct_generator!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = ServerOptions::parse();

    Ok(())
}
