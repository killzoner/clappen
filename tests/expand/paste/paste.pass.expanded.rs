pub struct NestedServerConfig {
    pub inner: ServerConfig,
    pub name: String,
}
pub struct ServerConfig {
    pub url: String,
    pub say_hello: bool,
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
fn main() {}
