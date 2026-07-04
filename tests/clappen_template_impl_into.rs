//! `Into<Base> for Prefixed` templates, flat and nested (`Into` needs the clippy allow).

mod into_flat {
    #[clappen::clappen(export = prefixed_struct_generator)]
    mod prefixed_struct_generator {
        #[derive(Debug, PartialEq)]
        struct ServerOptions {
            url: String,
            say_hello: Option<bool>,
        }

        #[clappen_template_impl]
        #[allow(clippy::from_over_into)]
        impl Into<Base> for Prefixed {
            fn into(self) -> Base {
                Base {
                    url: self.url,
                    say_hello: self.say_hello,
                }
            }
        }
    }

    prefixed_struct_generator!();
    prefixed_struct_generator!("test1");

    #[test]
    fn into_flat() {
        let test1 = Test1ServerOptions {
            test1_url: String::from("a"),
            test1_say_hello: Some(true),
        };
        let base: ServerOptions = test1.into();
        assert_eq!(
            base,
            ServerOptions {
                url: String::from("a"),
                say_hello: Some(true),
            }
        );
    }
}

mod into_nested {
    #[clappen::clappen(export = nested)]
    mod nested {
        #[derive(Debug, PartialEq)]
        pub struct MyStruct {
            pub id: String,
        }

        #[clappen_template_impl]
        #[allow(clippy::from_over_into)]
        impl Into<Base> for Prefixed {
            fn into(self) -> Base {
                Base { id: self.id }
            }
        }
    }

    #[clappen::clappen(export = options)]
    mod options {
        #[derive(Debug, PartialEq)]
        pub struct ServerOptions {
            pub url: String,
            #[clappen_command(apply = nested, prefix = "test")]
            pub nested: MyStruct,
        }

        #[clappen_template_impl]
        #[allow(clippy::from_over_into)]
        impl Into<Base> for Prefixed {
            fn into(self) -> Base {
                Base {
                    url: self.url,
                    nested: self.nested.into(),
                }
            }
        }
    }

    options!();
    options!("test1");

    #[test]
    fn into_nested() {
        let value = Test1ServerOptions {
            test1_url: String::from("hi"),
            test1_nested: __inner_test1_nested::Test1TestMyStruct {
                test1_test_id: String::from("7"),
            },
        };
        let base: ServerOptions = value.into();
        assert_eq!(
            base,
            ServerOptions {
                url: String::from("hi"),
                nested: __inner_nested::TestMyStruct {
                    test_id: String::from("7")
                },
            }
        );
    }
}

// two flatten levels: the conversion reaches the grandchild
mod into_deep_nested {
    #[clappen::clappen(export = nested3)]
    mod nested3 {
        #[derive(Debug, PartialEq)]
        pub struct Nested3 {
            pub id: String,
        }

        #[clappen_template_impl]
        #[allow(clippy::from_over_into)]
        impl Into<Base> for Prefixed {
            fn into(self) -> Base {
                Base { id: self.id }
            }
        }
    }

    #[clappen::clappen(export = nested2)]
    mod nested2 {
        #[derive(Debug, PartialEq)]
        pub struct Nested2 {
            pub url: String,
            #[clappen_command(apply = nested3, prefix = "test2")]
            pub nested: Nested3,
        }

        #[clappen_template_impl]
        #[allow(clippy::from_over_into)]
        impl Into<Base> for Prefixed {
            fn into(self) -> Base {
                Base {
                    url: self.url,
                    nested: self.nested.into(),
                }
            }
        }
    }

    #[clappen::clappen(export = nested1)]
    mod nested1 {
        #[derive(Debug, PartialEq)]
        pub struct Nested1 {
            pub url: String,
            #[clappen_command(apply = nested2, prefix = "test1")]
            pub nested: Nested2,
        }

        #[clappen_template_impl]
        #[allow(clippy::from_over_into)]
        impl Into<Base> for Prefixed {
            fn into(self) -> Base {
                Base {
                    url: self.url,
                    nested: self.nested.into(),
                }
            }
        }
    }

    nested1!();
    nested1!("level1");

    #[test]
    fn into_deep_nested() {
        let value = Level1Nested1 {
            level1_url: String::from("h"),
            level1_nested: __inner_level1_nested::Level1Test1Nested2 {
                level1_test1_url: String::from("d"),
                level1_test1_nested:
                    __inner_level1_nested::__inner_level1_test1_nested::Level1Test1Test2Nested3 {
                        level1_test1_test2_id: String::from("9"),
                    },
            },
        };
        let base: Nested1 = value.into();
        assert_eq!(
            base,
            Nested1 {
                url: String::from("h"),
                nested: __inner_nested::Test1Nested2 {
                    test1_url: String::from("d"),
                    test1_nested: __inner_nested::__inner_test1_nested::Test1Test2Nested3 {
                        test1_test2_id: String::from("9")
                    },
                },
            }
        );
    }
}
