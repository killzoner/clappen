use clap::Parser;
pub struct NestedServerConfig {
    pub inner: ServerConfig,
    pub name: String,
}
#[automatically_derived]
impl ::core::fmt::Debug for NestedServerConfig {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "NestedServerConfig",
            "inner",
            &self.inner,
            "name",
            &&self.name,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for NestedServerConfig {
    #[inline]
    fn clone(&self) -> NestedServerConfig {
        NestedServerConfig {
            inner: ::core::clone::Clone::clone(&self.inner),
            name: ::core::clone::Clone::clone(&self.name),
        }
    }
}
pub struct ServerConfig {
    pub url: String,
    pub say_hello: bool,
}
#[automatically_derived]
impl ::core::fmt::Debug for ServerConfig {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "ServerConfig",
            "url",
            &self.url,
            "say_hello",
            &&self.say_hello,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for ServerConfig {
    #[inline]
    fn clone(&self) -> ServerConfig {
        ServerConfig {
            url: ::core::clone::Clone::clone(&self.url),
            say_hello: ::core::clone::Clone::clone(&self.say_hello),
        }
    }
}
pub struct ServerOptions {
    /// Address to connect to.
    ///
    server_url: String,
    /// Do you need to say hello?.
    ///
    server_say_hello: Option<bool>,
}
impl ServerOptions {
    pub fn new_config(&self) -> crate::ServerConfig {
        crate::ServerConfig {
            url: self.server_url.clone(),
            say_hello: self.server_say_hello.unwrap_or_default(),
        }
    }
}
mod __inner_nested_struct_options {
    pub struct NestedServerOptions {
        /// Address to connect to.
        ///
        nested_url: String,
        /// Do you need to say hello?.
        ///
        nested_say_hello: Option<bool>,
    }
    impl NestedServerOptions {
        pub fn new_config(&self) -> crate::ServerConfig {
            crate::ServerConfig {
                url: self.nested_url.clone(),
                say_hello: self.nested_say_hello.unwrap_or_default(),
            }
        }
    }
}
pub struct NestedStructOptions {
    nested_name: String,
    nested_server: __inner_nested_struct_options::NestedServerOptions,
}
impl NestedStructOptions {
    pub fn new_nested_config(&self) -> crate::NestedServerConfig {
        NestedServerConfig {
            inner: self.nested_server.new_config(),
            name: self.nested_name.clone(),
        }
    }
}
mod __inner_bis_nested_struct_options {
    pub struct BisServerOptions {
        /// Address to connect to.
        ///
        bis_url: String,
        /// Do you need to say hello?.
        ///
        bis_say_hello: Option<bool>,
    }
    impl BisServerOptions {
        pub fn new_config(&self) -> crate::ServerConfig {
            crate::ServerConfig {
                url: self.bis_url.clone(),
                say_hello: self.bis_say_hello.unwrap_or_default(),
            }
        }
    }
}
pub struct BisNestedStructOptions {
    bis_name: String,
    bis_server: __inner_bis_nested_struct_options::BisServerOptions,
}
impl BisNestedStructOptions {
    pub fn new_nested_config(&self) -> crate::NestedServerConfig {
        NestedServerConfig {
            inner: self.bis_server.new_config(),
            name: self.bis_name.clone(),
        }
    }
}
mod __inner_third_nested_struct_options {
    pub struct ThirdServerOptions {
        /// Address to connect to.
        ///
        third_url: String,
        /// Do you need to say hello?.
        ///
        third_say_hello: Option<bool>,
    }
    impl ThirdServerOptions {
        pub fn new_config(&self) -> crate::ServerConfig {
            crate::ServerConfig {
                url: self.third_url.clone(),
                say_hello: self.third_say_hello.unwrap_or_default(),
            }
        }
    }
}
pub struct ThirdNestedStructOptions {
    third_name: String,
    third_server: __inner_third_nested_struct_options::ThirdServerOptions,
}
impl ThirdNestedStructOptions {
    pub fn new_nested_config(&self) -> crate::NestedServerConfig {
        NestedServerConfig {
            inner: self.third_server.new_config(),
            name: self.third_name.clone(),
        }
    }
}
#[command(version, about)]
struct Cli {
    #[command(flatten)]
    config: NestedStructOptions,
    #[command(flatten)]
    cli_bis: BisNestedStructOptions,
    #[command(flatten)]
    cli_third: ThirdNestedStructOptions,
}
#[automatically_derived]
#[allow(unused_qualifications, clippy::redundant_locals)]
impl clap::Parser for Cli {}
#[allow(
    dead_code,
    unreachable_code,
    unused_variables,
    unused_braces,
    unused_qualifications,
)]
#[allow(
    clippy::style,
    clippy::complexity,
    clippy::pedantic,
    clippy::restriction,
    clippy::perf,
    clippy::deprecated,
    clippy::nursery,
    clippy::cargo,
    clippy::suspicious_else_formatting,
    clippy::almost_swapped,
    clippy::redundant_locals,
)]
#[automatically_derived]
impl clap::CommandFactory for Cli {
    fn command<'b>() -> clap::Command {
        let __clap_app = clap::Command::new({
            let _ = "clappen-tests";
            "clappen-tests"
        });
        <Self as clap::Args>::augment_args(__clap_app)
    }
    fn command_for_update<'b>() -> clap::Command {
        let __clap_app = clap::Command::new({
            let _ = "clappen-tests";
            "clappen-tests"
        });
        <Self as clap::Args>::augment_args_for_update(__clap_app)
    }
}
#[allow(
    dead_code,
    unreachable_code,
    unused_variables,
    unused_braces,
    unused_qualifications,
)]
#[allow(
    clippy::style,
    clippy::complexity,
    clippy::pedantic,
    clippy::restriction,
    clippy::perf,
    clippy::deprecated,
    clippy::nursery,
    clippy::cargo,
    clippy::suspicious_else_formatting,
    clippy::almost_swapped,
    clippy::redundant_locals,
)]
#[automatically_derived]
impl clap::FromArgMatches for Cli {
    fn from_arg_matches(
        __clap_arg_matches: &clap::ArgMatches,
    ) -> ::std::result::Result<Self, clap::Error> {
        Self::from_arg_matches_mut(&mut __clap_arg_matches.clone())
    }
    fn from_arg_matches_mut(
        __clap_arg_matches: &mut clap::ArgMatches,
    ) -> ::std::result::Result<Self, clap::Error> {
        #![allow(deprecated)]
        let v = Cli {
            config: <NestedStructOptions as clap::FromArgMatches>::from_arg_matches_mut(
                __clap_arg_matches,
            )?,
            cli_bis: <BisNestedStructOptions as clap::FromArgMatches>::from_arg_matches_mut(
                __clap_arg_matches,
            )?,
            cli_third: <ThirdNestedStructOptions as clap::FromArgMatches>::from_arg_matches_mut(
                __clap_arg_matches,
            )?,
        };
        ::std::result::Result::Ok(v)
    }
    fn update_from_arg_matches(
        &mut self,
        __clap_arg_matches: &clap::ArgMatches,
    ) -> ::std::result::Result<(), clap::Error> {
        self.update_from_arg_matches_mut(&mut __clap_arg_matches.clone())
    }
    fn update_from_arg_matches_mut(
        &mut self,
        __clap_arg_matches: &mut clap::ArgMatches,
    ) -> ::std::result::Result<(), clap::Error> {
        #![allow(deprecated)]
        {
            #[allow(non_snake_case)]
            let config = &mut self.config;
            <NestedStructOptions as clap::FromArgMatches>::update_from_arg_matches_mut(
                config,
                __clap_arg_matches,
            )?;
        }
        {
            #[allow(non_snake_case)]
            let cli_bis = &mut self.cli_bis;
            <BisNestedStructOptions as clap::FromArgMatches>::update_from_arg_matches_mut(
                cli_bis,
                __clap_arg_matches,
            )?;
        }
        {
            #[allow(non_snake_case)]
            let cli_third = &mut self.cli_third;
            <ThirdNestedStructOptions as clap::FromArgMatches>::update_from_arg_matches_mut(
                cli_third,
                __clap_arg_matches,
            )?;
        }
        ::std::result::Result::Ok(())
    }
}
#[allow(
    dead_code,
    unreachable_code,
    unused_variables,
    unused_braces,
    unused_qualifications,
)]
#[allow(
    clippy::style,
    clippy::complexity,
    clippy::pedantic,
    clippy::restriction,
    clippy::perf,
    clippy::deprecated,
    clippy::nursery,
    clippy::cargo,
    clippy::suspicious_else_formatting,
    clippy::almost_swapped,
    clippy::redundant_locals,
)]
#[automatically_derived]
impl clap::Args for Cli {
    fn group_id() -> Option<clap::Id> {
        Some(clap::Id::from("Cli"))
    }
    fn augment_args<'b>(__clap_app: clap::Command) -> clap::Command {
        let __clap_app = __clap_app;
        let __clap_app = {
            let __clap_app = __clap_app
                .group(
                    clap::ArgGroup::new("Cli")
                        .multiple(true)
                        .args({
                            let members: [clap::Id; 0] = [];
                            members
                        }),
                );
            let __clap_app = __clap_app;
            let __clap_app = <NestedStructOptions as clap::Args>::augment_args(
                __clap_app,
            );
            let __clap_app = __clap_app;
            let __clap_app = <BisNestedStructOptions as clap::Args>::augment_args(
                __clap_app,
            );
            let __clap_app = __clap_app;
            let __clap_app = <ThirdNestedStructOptions as clap::Args>::augment_args(
                __clap_app,
            );
            __clap_app
        };
        __clap_app
            .version({
                let _ = "0.0.0";
                "0.0.0"
            })
    }
    fn augment_args_for_update<'b>(__clap_app: clap::Command) -> clap::Command {
        let __clap_app = __clap_app;
        let __clap_app = {
            let __clap_app = __clap_app
                .group(
                    clap::ArgGroup::new("Cli")
                        .multiple(true)
                        .args({
                            let members: [clap::Id; 0] = [];
                            members
                        }),
                );
            let __clap_app = __clap_app;
            let __clap_app = <NestedStructOptions as clap::Args>::augment_args_for_update(
                __clap_app,
            );
            let __clap_app = __clap_app;
            let __clap_app = <BisNestedStructOptions as clap::Args>::augment_args_for_update(
                __clap_app,
            );
            let __clap_app = __clap_app;
            let __clap_app = <ThirdNestedStructOptions as clap::Args>::augment_args_for_update(
                __clap_app,
            );
            __clap_app
        };
        __clap_app
            .version({
                let _ = "0.0.0";
                "0.0.0"
            })
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for Cli {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field3_finish(
            f,
            "Cli",
            "config",
            &self.config,
            "cli_bis",
            &self.cli_bis,
            "cli_third",
            &&self.cli_third,
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for Cli {
    #[inline]
    fn clone(&self) -> Cli {
        Cli {
            config: ::core::clone::Clone::clone(&self.config),
            cli_bis: ::core::clone::Clone::clone(&self.cli_bis),
            cli_third: ::core::clone::Clone::clone(&self.cli_third),
        }
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let config = cli.config.new_nested_config();
    let config_bis = cli.cli_bis.new_nested_config();
    let config_third = cli.cli_third.new_nested_config();
    {
        ::std::io::_print(format_args!("Config {0:?}\n", config));
    };
    {
        ::std::io::_print(format_args!("Config bis {0:?}\n", config_bis));
    };
    {
        ::std::io::_print(format_args!("Config third {0:?}\n", config_third));
    };
    Ok(())
}
