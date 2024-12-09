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
