/// Macros used for nested struct definition : []
struct ServerOptions {
    url: String,
    say_hello: Option<bool>,
}
/// Macros used for nested struct definition : []
/// Struct with prefix 'test', default_prefix: ''
struct TestServerOptions {
    test_url: String,
    test_say_hello: Option<bool>,
}
/// Template impl for `ServerOptions` (prefix 'test')
impl From<TestServerOptions> for ServerOptions {
    fn from(value: TestServerOptions) -> Self {
        Self {
            url: value.test_url,
            say_hello: value.test_say_hello,
        }
    }
}
fn main() {
    let test = TestServerOptions {
        test_url: String::from("Hello"),
        test_say_hello: Some(true),
    };
    let _: ServerOptions = test.into();
}
