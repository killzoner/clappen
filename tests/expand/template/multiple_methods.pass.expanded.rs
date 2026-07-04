trait Convert<T> {
    fn narrow(value: T) -> Self;
    fn widen(value: Self) -> T;
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
/// Template impl for `Options` (prefix 'test', nested via [])
impl Convert<TestOptions> for Options {
    fn narrow(value: TestOptions) -> Options {
        Options { url: value.test_url }
    }
    fn widen(value: Options) -> TestOptions {
        TestOptions { test_url: value.url }
    }
}
fn main() {
    let base = Options { url: String::from("x") };
    let prefixed: TestOptions = Options::widen(base);
    let _roundtrip: Options = Options::narrow(prefixed);
}
