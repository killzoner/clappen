// #[clappen::clappen(export = nested, gen_into)]
// mod generate {
//     #[derive(Debug, PartialEq)]
//     pub struct NestedStruct {
//         item1: u64,
//         item2: String,
//     }
// }

// #[clappen::clappen(export = generate, default_prefix = "default", gen_into)]
// mod generate {
//     #[derive(Debug, PartialEq)]
//     pub struct SimpleStruct {
//         #[clappen_command(apply = nested, prefix = "test1")]
//         item1: NestedStruct,
//         #[clappen_command(apply = nested, prefix = "test2")]
//         item2: NestedStruct,
//     }
// }

// generate!("one");
// generate!("two");

// #[test]
// fn into_nested_test() {
//     // let one = OneDefaultSimpleStruct {
//     //     one_default_item1: {
            
//     //     },
//     //     one_default_item2: String::from("Hello"),
//     // };
//     // let two = TwoDefaultSimpleStruct {
//     //     two_default_item1: 0,
//     //     two_default_item2: String::from("Hello"),
//     // };
//     // let compare = DefaultSimpleStruct {
//     //     default_item1: 0,
//     //     default_item2: String::from("Hello"),
//     // };
//     // let one_into: DefaultSimpleStruct = one.into();
//     // let two_into: DefaultSimpleStruct = two.into();
//     // assert_eq!(one_into, compare);
//     // assert_eq!(two_into, compare);
// }
