//! Validates cross-crate use of a generated macro and its `$crate` notation.

cross_crate_test::nested!();
cross_crate_test::nested!("test1");

cross_crate_test::prefixed_struct_generator!();
cross_crate_test::prefixed_struct_generator!("test1");

#[test]
fn prefixed_struct_and_impl_across_crates() {
    let nested = Test1MyStruct {
        test1_id: String::from("x"),
    };

    assert_eq!(nested.id(), "x");

    let base: MyStruct = nested.into();
    assert_eq!(
        base,
        MyStruct {
            id: String::from("x")
        }
    );
}

#[test]
fn flattened_child_across_crates() {
    let options = Test1ServerOptions {
        test1_nested: __inner_test1_nested::Test1TestMyStruct {
            test1_test_id: String::from("x"),
        },
    };

    let base: ServerOptions = options.into();
    assert_eq!(base.nested.test_id, "x");
}
