trait Convert<T> {
    fn narrow(value: T) -> Self;
    fn widen(value: Self) -> T;
}
/// Macros used for nested struct definition : []
/// Struct with prefix '', default_prefix: ''
struct ServerOptions {
    url: String,
}
/// Macros used for nested struct definition : []
/// Struct with prefix 'test', default_prefix: ''
struct TestServerOptions {
    test_url: String,
}
/// Template impl for `ServerOptions` (prefix 'test', nested via [])
impl Convert<TestServerOptions> for ServerOptions {
    fn narrow(value: TestServerOptions) -> ServerOptions {
        ServerOptions {
            url: value.test_url,
        }
    }
    fn widen(value: ServerOptions) -> TestServerOptions {
        TestServerOptions {
            test_url: value.url,
        }
    }
}
fn main() {
    let base = ServerOptions {
        url: String::from("x"),
    };
    let prefixed: TestServerOptions = ServerOptions::widen(base);
    let _roundtrip: ServerOptions = ServerOptions::narrow(prefixed);
}
