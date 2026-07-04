pub(crate) mod __inner_nested {
    /// Macros used for nested struct definition : []
    /// Struct with prefix 'Test', default_prefix: ''
    pub struct TestMyStruct {
        pub test_id: String,
    }
}
/// Macros used for nested struct definition : [nested]
/// Struct with prefix '', default_prefix: ''
pub struct ServerOptions {
    pub url: String,
    pub nested: __inner_nested::TestMyStruct,
}
pub(crate) mod __inner_test1_nested {
    /// Macros used for nested struct definition : []
    /// Struct with prefix 'Test1Test', default_prefix: ''
    pub struct Test1TestMyStruct {
        pub test1_test_id: String,
    }
}
/// Macros used for nested struct definition : [nested]
/// Struct with prefix 'test1', default_prefix: ''
pub struct Test1ServerOptions {
    pub test1_url: String,
    pub test1_nested: __inner_test1_nested::Test1TestMyStruct,
}
/// Template impl for `ServerOptions` (prefix 'test1', nested via [])
impl From<Test1ServerOptions> for ServerOptions {
    fn from(value: Test1ServerOptions) -> Self {
        Self {
            url: value.test1_url,
            nested: value.test1_nested.into(),
        }
    }
}
/// Template impl for `MyStruct` (prefix 'test1', nested via [test])
impl From<__inner_test1_nested::Test1TestMyStruct> for __inner_nested::TestMyStruct {
    fn from(value: __inner_test1_nested::Test1TestMyStruct) -> Self {
        Self {
            test_id: value.test1_test_id,
        }
    }
}
fn main() {
    let value = Test1ServerOptions {
        test1_url: String::from("hi"),
        test1_nested: __inner_test1_nested::Test1TestMyStruct {
            test1_test_id: String::from("x"),
        },
    };
    let _: ServerOptions = value.into();
}
