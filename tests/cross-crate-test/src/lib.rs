//! Defines a clappen macro that `tests/cross_crate.rs` invokes from a different crate.

#[clappen::clappen(export = nested)]
mod nested {
    pub struct MyStruct {
        pub id: String,
    }

    impl MyStruct {
        pub fn id(&self) -> &str {
            &self.id
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
}
