use clap::Parser;

#[clappen::clappen(export = log, default_prefix = "log")]
mod log {
    #[derive(clap::Args, Debug, Clone)]
    pub struct Level {
        #[arg(env, long)]
        pub level: String,
    }
}

#[clappen::clappen(export = prefixed_struct_generator, default_prefix = "my_prefix")]
mod m1 {
    /// A test command line
    #[derive(clap::Parser, Debug, Clone)]
    #[command(version)]
    pub struct ServerOptions {
        // Log level without default prefix
        //
        #[command(flatten)]
        #[clappen_command(apply = log)]
        log1: LogLevel,

        // Log level with default prefix
        //
        #[command(flatten)]
        #[clappen_command(apply = log, prefix = "log_prefix")]
        log2: LogLevel,
    }
}

prefixed_struct_generator!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = MyPrefixServerOptions::parse();

    Ok(())
}
