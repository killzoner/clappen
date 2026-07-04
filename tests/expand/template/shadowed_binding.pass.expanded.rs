struct Fallback {
    url: String,
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
        let url = value.test_url;
        let value = Fallback {
            url: String::from("fallback"),
        };
        Self {
            url: if url.is_empty() { value.url } else { url },
        }
    }
}
fn main() {
    let test = TestServerOptions {
        test_url: String::from("Hello"),
    };
    let options: ServerOptions = test.into();
    {
        match (&options.url, &"Hello") {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
}
