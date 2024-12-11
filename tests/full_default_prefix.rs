// cargo expand --test full_default_prefix
#[allow(dead_code)]
#[cfg(test)]
mod tests {
    #[clappen::clappen(export = nested)]
    mod nested {
        pub struct MyStruct {}
    }

    #[clappen::clappen(export = log, default_prefix = "log")]
    mod log {
        pub struct Level {
            pub level: String,
        }

        impl Level {
            pub fn another_nested_function(&self) -> String {
                self.level.clone()
            }
        }
    }

    nested!(); // define default nested struct

    #[clappen::clappen(export = prefixed_struct_generator, default_prefix="my_prefix")]
    mod m1 {
        pub struct ServerOptions {
            /// Address to connect to.
            ///
            url: String,

            /// Do you need to say hello?.
            ///
            say_hello: Option<bool>,

            // Log level with default prefix
            //
            #[clappen_command(apply = log)]
            log: LogLevel,

            /// A nested struct without a prefix.
            ///
            nested_default: MyStruct,

            #[clappen_command(apply = nested, prefix = "test")]
            nested: MyStruct,

            /// A nested struct that needs another prefix.
            ///
            #[clappen_command(apply = nested, prefix = "test1")]
            nested1: MyStruct,

            /// A nested struct that needs yet another prefix.
            #[clappen_command(apply = nested, prefix = "test2")]
            nested2: MyStruct,
        }

        impl ServerOptions {
            fn a_function(&self) -> String {
                format!("url: {}, say_hello: {:?}", self.url, self.say_hello)
            }
            fn another_function(&self) -> String {
                self.log.another_nested_function()
            }
        }

        impl ServerOptions {
            fn a_third_function_in_second_impl_block(&self) {}
        }
    }

    #[test]
    fn it_works() {
        prefixed_struct_generator!();
        prefixed_struct_generator!("second");

        let a = MyPrefixServerOptions {
            my_prefix_log: __inner_my_prefix_log::MyPrefixLogLevel {
                my_prefix_log_level: "warn".into(),
            },
            my_prefix_url: "url a".into(),
            my_prefix_say_hello: Some(false),
            my_prefix_nested_default: MyStruct {},
            my_prefix_nested: __inner_my_prefix_nested::MyPrefixTestMyStruct {},
            my_prefix_nested1: __inner_my_prefix_nested1::MyPrefixTest1MyStruct {},
            my_prefix_nested2: __inner_my_prefix_nested2::MyPrefixTest2MyStruct {},
        };

        assert_eq!(a.a_function(), "url: url a, say_hello: Some(false)");
        assert_eq!(a.another_function(), "warn");
        a.a_third_function_in_second_impl_block();

        let b = SecondMyPrefixServerOptions {
            second_my_prefix_log: __inner_second_my_prefix_log::SecondMyPrefixLogLevel {
                second_my_prefix_log_level: "info".into(),
            },
            second_my_prefix_url: "url b".into(),
            second_my_prefix_say_hello: Some(true),
            second_my_prefix_nested_default: MyStruct {},
            second_my_prefix_nested: __inner_second_my_prefix_nested::SecondMyPrefixTestMyStruct {},
            second_my_prefix_nested1:
                __inner_second_my_prefix_nested1::SecondMyPrefixTest1MyStruct {},
            second_my_prefix_nested2:
                __inner_second_my_prefix_nested2::SecondMyPrefixTest2MyStruct {},
        };

        assert_eq!(b.a_function(), "url: url b, say_hello: Some(true)");
        assert_eq!(b.another_function(), "info");
        b.a_third_function_in_second_impl_block();
    }
}
