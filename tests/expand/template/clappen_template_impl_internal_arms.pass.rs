#![allow(dead_code)]

// Calls the internal `@__struct` and `@__template` arms directly.
#[clappen::clappen(export = nested)]
mod nested {
    pub struct MyStruct {
        pub id: String,
    }

    #[clappen_template_impl]
    impl From<Prefixed> for Base {
        fn from(value: Prefixed) -> Self {
            Self { id: value.id }
        }
    }
}

nested!(); // base `MyStruct`, the `From` target

// `@__struct`: the prefixed struct, no template impl
nested!(@__struct "test");

// `@__struct` then `@__template` (empty chain) == what `nested!("test1")` expands to
nested!(@__struct "test1");
nested!(@__template "test1", chain = []);

nested!("test2"); // public prefixed arm, for comparison

// The four chain step shapes. Each step names a real module, so a matcher that no longer fits the
// emitter fails here.
mod __inner_nested {
    nested!(@__struct); // no command prefix
    nested!(@__struct "Bp"); // command prefix only
}
mod __inner_pd_nested {
    nested!(@__struct "Pd"); // parent default_prefix only
}
mod __inner_pd4_nested {
    nested!(@__struct "Pd4Dp"); // both slots
}
mod __inner_c4_pd4_nested {
    nested!(@__struct "C4Pd4Dp"); // both slots, one level under the "c4" prefix
}

// no prefix: `Base` is the plain `MyStruct` and `Prefixed` walks the chain
nested!(@__template chain = [(nested)]);
nested!(@__template chain = [("bp", nested)]);
nested!(@__template chain = [(nested, "pd"),]); // trailing comma
nested!(@__template chain = [("dp", nested, "pd4")]);

// a prefix and a chain together: both ends walk the chain
nested!(@__template "c4", chain = [("dp", nested, "pd4")]);

fn main() {}
