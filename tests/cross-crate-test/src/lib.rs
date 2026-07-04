//! Defines a clappen macro that `tests/cross_crate.rs` invokes from a different crate.

#[clappen::clappen(export = nested)]
mod nested {
    #[derive(Debug, PartialEq)]
    pub struct MyStruct {
        pub id: String,
    }

    impl MyStruct {
        pub fn id(&self) -> &str {
            &self.id
        }
    }

    #[clappen_template_impl]
    impl From<Prefixed> for Base {
        fn from(value: Prefixed) -> Self {
            Self { id: value.id }
        }
    }

    #[clappen_template_impl]
    impl From<Base> for Prefixed {
        fn from(value: Base) -> Self {
            Self { id: value.id }
        }
    }
}

#[clappen::clappen(export = prefixed_struct_generator)]
mod m1 {
    pub struct ServerOptions {
        // `apply` must resolve in the caller's crate, so the path starts with `$crate`
        #[clappen_command(apply = $crate::nested, prefix = "test")]
        pub nested: MyStruct,
    }

    #[clappen_template_impl]
    impl From<Prefixed> for Base {
        fn from(value: Prefixed) -> Self {
            Self {
                nested: MyStruct::from(value.nested).into(),
            }
        }
    }
}
