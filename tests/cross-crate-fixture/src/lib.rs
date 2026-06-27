//! Defines a clappen macro so `tests/cross_crate.rs` can invoke it from a *different* crate. The
//! generated macro must not refer to itself by name (a prefixed `greeter!("x")` used to), which
//! fails only when the macro is invoked from outside the crate that defined it.

#[clappen::clappen(export = greeter)]
mod greeter {
    #[derive(Debug, PartialEq)]
    pub struct Greeter {
        pub name: String,
    }

    #[clappen_template_impl]
    impl From<Prefixed> for Base {
        fn from(value: Prefixed) -> Self {
            Self { name: value.name }
        }
    }
}
