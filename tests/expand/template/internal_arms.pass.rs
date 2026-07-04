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

// The four ways a `@__template` chain step can fill its two optional slots. A step addresses a real
// module, so a matcher that drifts from the emitter fails here instead of deep in a macro error.
mod __inner_nested {
    remote!(@__struct); // no command prefix
    remote!(@__struct "Bp"); // command prefix only
}
mod __inner_pd_nested {
    remote!(@__struct "Pd"); // parent default_prefix only
}
mod __inner_pd4_nested {
    remote!(@__struct "Pd4Dp"); // both slots
}
mod __inner_c4_pd4_nested {
    remote!(@__struct "C4Pd4Dp"); // both slots, one level under the "c4" prefix
}

// no prefix: `Base` is the plain `Remote` and `Prefixed` walks the chain
remote!(@__template chain = [(nested)]);
remote!(@__template chain = [("bp", nested)]);
remote!(@__template chain = [(nested, "pd"),]); // trailing comma
remote!(@__template chain = [("dp", nested, "pd4")]);

// a prefix and a chain together: both ends walk the chain
remote!(@__template "c4", chain = [("dp", nested, "pd4")]);

fn main() {}
