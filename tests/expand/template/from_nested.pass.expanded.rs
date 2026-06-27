pub(crate) mod __inner_remote {
    /// Macros used for nested struct definition : []
    /// Struct with prefix 'Test', default_prefix: ''
    pub struct TestRemote {
        pub test_id: String,
    }
}
/// Macros used for nested struct definition : [remote]
pub struct ServerOptions {
    pub url: String,
    pub remote: __inner_remote::TestRemote,
}
pub(crate) mod __inner_test1_remote {
    /// Macros used for nested struct definition : []
    /// Struct with prefix 'Test1Test', default_prefix: ''
    pub struct Test1TestRemote {
        pub test1_test_id: String,
    }
}
/// Macros used for nested struct definition : [remote]
/// Struct with prefix 'test1', default_prefix: ''
pub struct Test1ServerOptions {
    pub test1_url: String,
    pub test1_remote: __inner_test1_remote::Test1TestRemote,
}
/// Template impl for `ServerOptions` (prefix 'test1')
impl From<Test1ServerOptions> for ServerOptions {
    fn from(value: Test1ServerOptions) -> Self {
        Self {
            url: value.test1_url,
            remote: value.test1_remote.into(),
        }
    }
}
/// Template impl for `Remote` (prefix 'test1', nested via test)
impl From<__inner_test1_remote::Test1TestRemote> for __inner_remote::TestRemote {
    fn from(value: __inner_test1_remote::Test1TestRemote) -> Self {
        Self {
            test_id: value.test1_test_id,
        }
    }
}
fn main() {
    let value = Test1ServerOptions {
        test1_url: String::from("hi"),
        test1_remote: __inner_test1_remote::Test1TestRemote {
            test1_test_id: String::from("x"),
        },
    };
    let _: ServerOptions = value.into();
}
