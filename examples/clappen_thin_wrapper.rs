use clap::Parser;

#[clappen::clappen(export = nested)]
mod nested {
    #[derive(clap::Args, Debug, Clone)]
    pub struct Remote {
        #[arg(env, long)]
        id: String,
    }
}

nested!("test"); // define default nested struct
nested!("test1");

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
    nested: TestRemote,

    /// A nested struct that needs another prefix.
    ///
    #[command(flatten)]
    nested1: Test1Remote,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = ServerOptions::parse();

    Ok(())
}
