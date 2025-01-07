#![allow(dead_code)]

#[clappen::clappen(export = generate, gen_into)]
mod generate {
    #[derive(Debug, PartialEq)]
    struct SimpleStruct {
        item1: u64,
        item2: String,
    }
}

generate!("one");
generate!("two");

#[test]
fn into_test() {
    let one = OneSimpleStruct {
        one_item1: 0,
        one_item2: String::from("Hello"),
    };
    let two = TwoSimpleStruct {
        two_item1: 0,
        two_item2: String::from("Hello"),
    };
    let compare = SimpleStruct {
        item1: 0,
        item2: String::from("Hello"),
    };
    let one_into: SimpleStruct = one.into();
    let two_into: SimpleStruct = two.into();
    assert_eq!(one_into, compare);
    assert_eq!(two_into, compare);
}
