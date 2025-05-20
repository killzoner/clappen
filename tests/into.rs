#![allow(dead_code)]

use std::convert::From;
#[clappen::clappen(export = generate)]
mod generate {
    #[derive(Debug, PartialEq)]
    struct SimpleStruct {
        item1: u64,
        item2: String,
    }

    #[clappen::clappen_impl_custom(ignore_self = "SimpleStruct")]
    #[allow(clippy::from_over_into)]
    impl Into<SimpleStruct> for SimpleStruct {
        fn into(self) -> SimpleStruct {
            SimpleStruct {
                item1: self.item1,
                item2: self.item2,
            }
        }
    }
}

generate!();
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

