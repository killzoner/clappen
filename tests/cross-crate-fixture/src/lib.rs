//! Defines a clappen macro so `tests/cross_crate.rs` can invoke it from a *different* crate. The
//! generated macro must not refer to itself by name (a prefixed `remote!("x")` used to), which
//! fails only when the macro is invoked from outside the crate that defined it.

#[clappen::clappen(export = remote)]
mod remote {
    #[derive(Debug, PartialEq)]
    pub struct Remote {
        pub id: String,
    }

    #[clappen_template_impl]
    impl From<Prefixed> for Base {
        fn from(value: Prefixed) -> Self {
            Self { id: value.id }
        }
    }
}

// A parent that flattens the templated `remote` child, reached cross-crate. `apply` must name a
// path that resolves in the caller's crate, so it is spelled `$crate::remote` here.
#[clappen::clappen(export = holder)]
mod holder {
    #[derive(Debug, PartialEq)]
    pub struct Holder {
        pub label: String,

        #[clappen_command(apply = $crate::remote, prefix = "inner")]
        pub remote: Remote,
    }

    #[clappen_template_impl]
    impl From<Prefixed> for Base {
        fn from(value: Prefixed) -> Self {
            Self {
                label: value.label,
                remote: value.remote.into(),
            }
        }
    }
}
