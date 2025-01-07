#[clappen::clappen(export = generate, gen_into)]
mod generate {
    struct SimpleStruct {
        item1: u64,
        item2: String,
    }
}

generate!("one");
generate!("two");

fn main() {
    let one = OneSimpleStruct {
        one_item1: 0,
        one_item2: String::from("Hello"),
    };
    let two = TwoSimpleStruct {
        two_item1: 0,
        two_item2: String::from("Hello"),
    };
    let _: SimpleStruct = one.into();
    let _: SimpleStruct = two.into();
}
