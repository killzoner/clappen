//! `From<Prefixed> for Base` templates: with a default_prefix, and nested.

mod from_default_prefix {
    #[clappen::clappen(export = log, default_prefix = "log")]
    mod log {
        #[derive(Debug, PartialEq)]
        struct Level {
            level: String,
        }

        #[clappen_template_impl]
        impl From<Prefixed> for Base {
            fn from(value: Prefixed) -> Self {
                Self { level: value.level }
            }
        }
    }

    log!();
    log!("level1");

    #[test]
    fn from_default_prefix() {
        let level1 = Level1LogLevel {
            level1_log_level: String::from("warn"),
        };
        assert_eq!(
            LogLevel::from(level1),
            LogLevel {
                log_level: String::from("warn"),
            }
        );
    }
}

#[clappen::clappen(export = nested)]
mod nested {
    #[derive(Debug, PartialEq)]
    pub struct MyStruct {
        pub id: String,
    }

    #[clappen_template_impl]
    impl From<Prefixed> for Base {
        fn from(value: Prefixed) -> Self {
            Self { id: value.id }
        }
    }

    // a child flattened in a parent with a template also converts back from its root
    #[clappen_template_impl]
    impl From<Base> for Prefixed {
        fn from(value: Base) -> Self {
            Self { id: value.id }
        }
    }
}

// the root `MyStruct`, which every conversion of a `MyStruct` copy goes through
nested!();

// the parent's default_prefix also prefixes the flattened child's type and fields
mod from_nested_under_default_prefix {
    use super::MyStruct;

    #[clappen::clappen(export = api, default_prefix = "api")]
    mod api {
        #[derive(Debug, PartialEq)]
        pub struct ServerOptions {
            pub url: String,
            #[clappen_command(apply = nested, prefix = "test")]
            pub nested: MyStruct,
        }

        #[clappen_template_impl]
        impl From<Prefixed> for Base {
            fn from(value: Prefixed) -> Self {
                Self {
                    url: value.url,
                    nested: MyStruct::from(value.nested).into(),
                }
            }
        }
    }

    api!();
    api!("main");

    #[test]
    fn from_nested_under_default_prefix() {
        let value = MainApiServerOptions {
            main_api_url: String::from("hi"),
            main_api_nested: __inner_main_api_nested::MainApiTestMyStruct {
                main_api_test_id: String::from("7"),
            },
        };
        assert_eq!(
            ApiServerOptions::from(value),
            ApiServerOptions {
                api_url: String::from("hi"),
                api_nested: __inner_api_nested::ApiTestMyStruct {
                    api_test_id: String::from("7")
                },
            }
        );
    }
}

// two flatten levels: the conversion reaches the grandchild
mod from_deep_nested {
    #[clappen::clappen(export = nested3)]
    mod nested3 {
        #[derive(Debug, PartialEq)]
        pub struct Nested3 {
            pub id: String,
        }

        #[clappen_template_impl]
        impl From<Prefixed> for Base {
            fn from(value: Prefixed) -> Self {
                Self { id: value.id }
            }
        }

        #[clappen_template_impl]
        impl From<Base> for Prefixed {
            fn from(value: Base) -> Self {
                Self { id: value.id }
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
        impl From<Prefixed> for Base {
            fn from(value: Prefixed) -> Self {
                Self {
                    url: value.url,
                    nested: Nested3::from(value.nested).into(),
                }
            }
        }

        #[clappen_template_impl]
        impl From<Base> for Prefixed {
            fn from(value: Base) -> Self {
                Self {
                    url: value.url,
                    nested: Nested3::from(value.nested).into(),
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
        impl From<Prefixed> for Base {
            fn from(value: Prefixed) -> Self {
                Self {
                    url: value.url,
                    nested: Nested2::from(value.nested).into(),
                }
            }
        }
    }

    // each root in its own module: `nested2!()` and `nested1!()` both define `__inner_nested`
    mod nested3_root {
        nested3!();
    }
    mod nested2_root {
        use super::nested3_root::Nested3;
        nested2!();
    }
    use nested2_root::Nested2;
    use nested3_root::Nested3;

    nested1!();
    nested1!("level1");

    #[test]
    fn from_deep_nested() {
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
        assert_eq!(
            Nested1::from(value),
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

// a parent without a template: its copy of a templated child still converts to the child's root
mod from_parent_without_template {
    use super::MyStruct;

    #[clappen::clappen(export = middle)]
    mod middle {
        #[derive(Debug, PartialEq)]
        pub struct Middle {
            pub url: String,
            #[clappen_command(apply = nested, prefix = "api")]
            pub api: MyStruct,
        }

        #[clappen_template_impl]
        impl From<Prefixed> for Base {
            fn from(value: Prefixed) -> Self {
                Self {
                    url: value.url,
                    api: MyStruct::from(value.api).into(),
                }
            }
        }
    }

    #[clappen::clappen(export = top)]
    mod top {
        pub struct Top {
            #[clappen_command(apply = middle, prefix = "x")]
            pub middle: Middle,
        }
    }

    middle!();
    top!();

    #[test]
    fn from_parent_without_template() {
        let top = Top {
            middle: __inner_middle::XMiddle {
                x_url: String::from("u"),
                x_api: __inner_middle::__inner_x_api::XApiMyStruct {
                    x_api_id: String::from("7"),
                },
            },
        };
        assert_eq!(
            Middle::from(top.middle),
            Middle {
                url: String::from("u"),
                api: __inner_api::ApiMyStruct {
                    api_id: String::from("7")
                },
            }
        );
    }
}

// a parent without a template still gets its flattened children's `From` impls
mod flattened_fields_map_to_base {
    use super::MyStruct;

    #[clappen::clappen(export = options)]
    mod options {
        #[derive(Debug)]
        pub struct ServerOptions {
            #[clappen_command(apply = nested, prefix = "test1")]
            pub nested1: MyStruct,
            #[clappen_command(apply = nested, prefix = "test2")]
            pub nested2: MyStruct,
        }
    }

    options!();

    #[test]
    fn flattened_fields_map_to_base() {
        let options = ServerOptions {
            nested1: __inner_nested1::Test1MyStruct {
                test1_id: String::from("h"),
            },
            nested2: __inner_nested2::Test2MyStruct {
                test2_id: String::from("h"),
            },
        };

        let nested1: MyStruct = options.nested1.into();
        let nested2: MyStruct = options.nested2.into();

        assert_eq!(nested1, nested2);
        assert_eq!(
            nested1,
            MyStruct {
                id: String::from("h"),
            }
        );
    }
}
