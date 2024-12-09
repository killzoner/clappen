use clap::{Parser, Subcommand};

#[clappen::clappen(export = nested)]
mod nested {
    #[derive(clap::Args, Debug, Clone)]
    pub struct Remote {
        #[arg(env, long)]
        id: String,
    }
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// A sub command
    MySubCommand { name: Option<String> },
}

use nested as local_nested;

#[clappen::clappen(export = prefixed_struct_generator)]
mod m1 {
    /// A test command line
    #[derive(clap::Parser, Debug, Clone)]
    #[command(version, about)]
    pub struct ServerOptions {
        /// Address to connect to.
        ///
        #[arg(env, long)]
        url: String,

        /// Do you need to say hello?.
        ///
        #[arg(env, long)]
        say_hello: Option<bool>,

        #[command(subcommand)]
        subcommand: Commands,

        /// A nested struct that needs a prefix.
        ///
        #[command(flatten)]
        #[clappen_command(apply = $crate::local_nested)]
        nested_default: Remote,

        /// A nested struct that needs a prefix.
        ///
        #[command(flatten)]
        #[clappen_command(apply = $crate::local_nested, prefix = "test")]
        // disambiguous macro notation
        nested: super::Remote, // fully qualified type

        /// A nested struct that needs another prefix.
        ///
        #[command(flatten)]
        #[clappen_command(apply = nested, prefix = "test1")]
        nested1: Remote,
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
