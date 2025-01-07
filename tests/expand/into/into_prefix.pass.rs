#[clappen::clappen(export = generate, default_prefix = "default", gen_into)]
mod generate {
    struct SimpleStruct {
        item1: u64,
        item2: String,
    }
}

generate!("one");
generate!("two");

fn main() {
    let one = OneDefaultSimpleStruct {
        one_default_item1: 0,
        one_default_item2: String::from("Hello"),
    };
    let two = TwoDefaultSimpleStruct {
        two_default_item1: 0,
        two_default_item2: String::from("Hello"),
    };
    let _: DefaultSimpleStruct = one.into();
    let _: DefaultSimpleStruct = two.into();
}
