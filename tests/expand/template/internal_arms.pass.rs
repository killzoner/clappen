#![allow(dead_code)]

// Pins the internal `@__struct` and `@__template` arms directly, rather than only through the
// `clappen_command` nesting and the `("prefix")` self-call that normally reach them.
#[clappen::clappen(export = remote)]
mod remote {
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

remote!(); // base `Remote`, the `From` target

// `@__struct`: the prefixed struct, no template impl
remote!(@__struct "test");

// `@__struct` then `@__template` (empty chain) == what `remote!("test1")` expands to
remote!(@__struct "test1");
remote!(@__template "test1", chain = []);

remote!("test2"); // public prefixed arm, for comparison

fn main() {}
