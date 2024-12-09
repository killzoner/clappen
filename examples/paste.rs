use clap::Parser;

#[derive(Debug, Clone)]
pub struct NestedServerConfig {
    pub inner: ServerConfig,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub url: String,
    pub say_hello: bool,
}

// test with `cargo expand --example paste`
#[macro_export]
macro_rules! cli_server_config_with_prefix {
    () => {
        $crate::cli_server_config_with_prefix!(Server, Options);
    };
    ($prefix: ident) => {
        $crate::cli_server_config_with_prefix!($prefix, ServerOptions);
    };
    ($prefix: ident, $default_prefix: ident) => {
        paste::paste! {
                #[derive(clap::Args, Debug, Clone)]
                pub struct [<$prefix:camel $default_prefix:camel>] {
                    /// Address to connect to.
                    ///
                    #[arg(env, long, required = true)]
                    [<$prefix:snake:lower _url>]: String,

                    /// Do you need to say hello?.
                    ///
                    #[arg(
                        env,
                        long,
                        required = false,
                    )]
                    [<$prefix:snake:lower _say_hello>] : Option<bool>,
                }

            impl [<$prefix:camel $default_prefix:camel>] {
                pub fn new_config(&self) -> $crate::ServerConfig {
                    $crate::ServerConfig{
                        url: self.[<$prefix:snake:lower _url>].clone(),
                        say_hello: self.[<$prefix:snake:lower _say_hello>].unwrap_or_default(),
                    }
                }
            }
        }
    };
}

#[macro_export]
macro_rules! cli_nested_config_with_prefix {
    () => { $crate::cli_nested_config_with_prefix!(Nested, StructOptions); };
    ($prefix: ident) => { $crate::cli_nested_config_with_prefix!($prefix, NestedStructOptions); };
    ($prefix: ident, $default_prefix: ident) => {
        paste::paste! {
            mod [<__inner _$prefix:snake:lower _$default_prefix:snake:lower>] {
                $crate::cli_server_config_with_prefix!($prefix);
            }

            #[derive(clap::Args, Debug, Clone)]
            pub struct [<$prefix:camel $default_prefix:camel>] {
                #[arg(env, long, required = true)]
                [<$prefix:snake:lower _name>]: String,

                #[command(flatten)]
                [<$prefix:snake:lower _server>]:  [<__inner_ $prefix:snake:lower _$default_prefix:snake:lower>]::[<$prefix:camel ServerOptions>],
            }

            impl [<$prefix:camel $default_prefix:camel>] {
                pub fn new_nested_config(
                    &self,
                ) -> $crate::NestedServerConfig {
                    NestedServerConfig {
                        inner: self.[<$prefix:snake:lower _server>].new_config(),
                        name: self.[<$prefix:snake:lower _name>].clone(),
                    }
                }
            }
        }
    };
}

cli_server_config_with_prefix!();
cli_nested_config_with_prefix!();

// define second nested config struct with prefix Bis.
cli_nested_config_with_prefix!(Bis);

// define second nested config struct with prefix Bis.
cli_nested_config_with_prefix!(Third);

#[derive(Parser, Debug, Clone)]
#[command(version)]
struct Cli {
    #[command(flatten)]
    config: NestedStructOptions,

    #[command(flatten)]
    cli_bis: BisNestedStructOptions,

    #[command(flatten)]
    cli_third: ThirdNestedStructOptions,
}

// cargo run --example paste -- --help
// cargo run --example paste -- --nested-name="nested name" --nested-url="nested url" --bis-name="bis name" --bis-url="bis url" --third-name="third name" --third-url="third url"
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let config = cli.config.new_nested_config();
    let config_bis = cli.cli_bis.new_nested_config();
    let config_third = cli.cli_third.new_nested_config();

    println!("Config {:?}", config);
    println!("Config bis {:?}", config_bis);
    println!("Config third {:?}", config_third);

    Ok(())
}
