//! Invokes `cross_crate_fixture::greeter` from this separate crate and converts a prefixed struct
//! back to the base. The cross-crate call is the point: it caught a "cannot find macro" failure
//! that no same-crate test could. See the fixture crate for the mechanism.

cross_crate_fixture::greeter!();
cross_crate_fixture::greeter!("remote");

#[test]
fn prefixed_instantiation_across_crates() {
    let remote = RemoteGreeter {
        remote_name: String::from("x"),
    };
    let base: Greeter = remote.into();
    assert_eq!(
        base,
        Greeter {
            name: String::from("x")
        }
    );
}
