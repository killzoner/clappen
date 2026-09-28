//! A user crate can set `#![deny(missing_docs)]`. The generated macro must have a doc comment, or
//! that crate does not compile.
#![deny(missing_docs)]

#[clappen::clappen(export = documented)]
mod documented {
    /// A struct.
    pub struct ServerOptions {
        /// A field.
        pub url: String,
    }
}

documented!();
documented!("test");

#[test]
fn prefixed_struct_under_deny_missing_docs() {
    let options = TestServerOptions {
        test_url: String::from("x"),
    };

    assert_eq!(options.test_url, "x");
}
