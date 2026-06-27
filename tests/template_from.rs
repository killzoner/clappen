//! `From<Prefixed> for Base` templates: flat, with a default_prefix, and nested.

#[clappen::clappen(export = prefixed_struct_generator)]
mod prefixed_struct_generator {
    #[derive(Debug, PartialEq)]
    struct ServerOptions {
        url: String,
        say_hello: Option<bool>,
    }

    // `Base` = the default struct, `Prefixed` = the generated variant
    #[clappen_template_impl]
    impl From<Prefixed> for Base {
        fn from(value: Prefixed) -> Self {
            Self {
                url: value.url,
                say_hello: value.say_hello,
            }
        }
    }
}

prefixed_struct_generator!();
prefixed_struct_generator!("test1");
prefixed_struct_generator!("test2");

#[test]
fn from_flat() {
    let base = ServerOptions {
        url: String::from("a"),
        say_hello: Some(true),
    };
    let test1 = Test1ServerOptions {
        test1_url: String::from("a"),
        test1_say_hello: Some(true),
    };
    let test2 = Test2ServerOptions {
        test2_url: String::from("a"),
        test2_say_hello: Some(true),
    };
    assert_eq!(ServerOptions::from(test1), base);
    assert_eq!(ServerOptions::from(test2), base);
}

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

#[clappen::clappen(export = remote)]
mod remote {
    #[derive(Debug, PartialEq)]
    pub struct Remote {
        pub id: String,
    }

    #[clappen_template_impl]
    impl From<Prefixed> for Base {
        fn from(value: Prefixed) -> Self {
            Self { id: value.id }
        }
    }
}

#[clappen::clappen(export = options)]
mod options {
    #[derive(Debug, PartialEq)]
    pub struct Options {
        pub url: String,
        #[clappen_command(apply = remote, prefix = "test")]
        pub remote: Remote,
    }

    // the nested field converts through its own template
    #[clappen_template_impl]
    impl From<Prefixed> for Base {
        fn from(value: Prefixed) -> Self {
            Self {
                url: value.url,
                remote: value.remote.into(),
            }
        }
    }
}

options!();
options!("test1");

#[test]
fn from_nested() {
    let value = Test1Options {
        test1_url: String::from("hi"),
        test1_remote: __inner_test1_remote::Test1TestRemote {
            test1_test_id: String::from("7"),
        },
    };
    assert_eq!(
        Options::from(value),
        Options {
            url: String::from("hi"),
            remote: __inner_remote::TestRemote {
                test_id: String::from("7")
            },
        }
    );
}

// a user type literally named `Base` would clash with the default tag; custom tag idents
// (`base`/`prefixed`) avoid it
#[derive(Debug, PartialEq)]
struct Base {
    other: u64,
}

#[clappen::clappen(export = my_struct)]
mod my_struct {
    #[derive(Debug, PartialEq)]
    struct MyStruct {
        url: String,
    }

    #[clappen_template_impl(base = Canonical, prefixed = Variant)]
    impl From<Variant> for Canonical {
        fn from(value: Variant) -> Self {
            Self { url: value.url }
        }
    }
}

my_struct!();
my_struct!("test");

#[test]
fn from_custom_tags() {
    // the unrelated `Base` type still exists and is untouched
    let _ = Base { other: 0 };
    let test = TestMyStruct {
        test_url: String::from("9"),
    };
    assert_eq!(
        MyStruct::from(test),
        MyStruct {
            url: String::from("9")
        }
    );
}

// two levels of flattening: a struct holding a child holding a grandchild.
// the conversion chain must reach the innermost struct.
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
                nested: value.nested.into(),
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
                nested: value.nested.into(),
            }
        }
    }
}

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

// the example's case: base `multi!()` coexists with several prefixed variants, each converted
// back to the base type and compared
#[clappen::clappen(export = config)]
mod config {
    #[derive(Debug, PartialEq)]
    struct Config {
        id: String,
    }

    #[clappen_template_impl]
    impl From<Prefixed> for Base {
        fn from(value: Prefixed) -> Self {
            Self { id: value.id }
        }
    }
}

config!();
config!("level1");
config!("level2");

#[test]
fn from_multi_prefix() {
    let level1: Config = Level1Config {
        level1_id: String::from("1"),
    }
    .into();
    let level2_same: Config = Level2Config {
        level2_id: String::from("1"),
    }
    .into();
    let level2_diff: Config = Level2Config {
        level2_id: String::from("2"),
    }
    .into();

    assert_eq!(
        Config {
            id: String::from("1")
        },
        level1
    );
    assert_eq!(level1, level2_same);
    assert_ne!(level1, level2_diff);
}
