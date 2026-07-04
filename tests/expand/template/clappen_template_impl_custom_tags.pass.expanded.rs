#![allow(dead_code)]
struct Base {
    other: u64,
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
impl From<TestServerOptions> for ServerOptions {
    fn from(value: TestServerOptions) -> Self {
        Self { url: value.test_url }
    }
}
fn main() {}
