#![allow(dead_code)]
struct Base {
    other: u64,
}
/// Macros used for nested struct definition : []
struct Options {
    url: String,
}
/// Macros used for nested struct definition : []
/// Struct with prefix 'test', default_prefix: ''
struct TestOptions {
    test_url: String,
}
/// Template impl for `Options` (prefix 'test')
impl From<TestOptions> for Options {
    fn from(value: TestOptions) -> Self {
        Self { url: value.test_url }
    }
}
fn main() {}
