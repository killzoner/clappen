//! Invokes `cross_crate_fixture::remote` from this separate crate and converts a prefixed struct
//! back to the base. The cross-crate call is the point: it caught a "cannot find macro" failure
//! that no same-crate test could. See the fixture crate for the mechanism.

cross_crate_fixture::remote!();
cross_crate_fixture::remote!("test1");

#[test]
fn prefixed_instantiation_across_crates() {
    let test1 = Test1Remote {
        test1_id: String::from("x"),
    };
    let base: Remote = test1.into();
    assert_eq!(
        base,
        Remote {
            id: String::from("x")
        }
    );
}

// Flattening a templated child cross-crate: `apply` must be a path that resolves in the caller's
// crate, so the fixture spells it `$crate::remote`. Invoking the parent is enough to expose a
// missing child conversion, because the parent's generated `From` body calls `.into()` on the child.
cross_crate_fixture::holder!();
cross_crate_fixture::holder!("test1");
