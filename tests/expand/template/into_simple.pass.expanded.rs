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
#[allow(clippy::from_over_into)]
impl Into<ServerOptions> for TestServerOptions {
    fn into(self) -> ServerOptions {
        ServerOptions {
            url: self.test_url,
            say_hello: self.test_say_hello,
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
