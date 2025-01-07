/// Macros used for nested struct definition : []
struct SimpleStruct {
    item1: u64,
    item2: String,
}
/// Macros used for nested struct definition : []
/// Struct with prefix 'one', default_prefix: ''
struct OneSimpleStruct {
    one_item1: u64,
    one_item2: String,
}
/// Fields with prefix: [item1,item2]
#[allow(clippy::from_over_into)]
impl Into<SimpleStruct> for OneSimpleStruct {
    fn into(self) -> SimpleStruct {
        let item1 = self.one_item1.into();
        let item2 = self.one_item2.into();
        SimpleStruct { item1, item2 }
    }
}
/// Macros used for nested struct definition : []
/// Struct with prefix 'two', default_prefix: ''
struct TwoSimpleStruct {
    two_item1: u64,
    two_item2: String,
}
/// Fields with prefix: [item1,item2]
#[allow(clippy::from_over_into)]
impl Into<SimpleStruct> for TwoSimpleStruct {
    fn into(self) -> SimpleStruct {
        let item1 = self.two_item1.into();
        let item2 = self.two_item2.into();
        SimpleStruct { item1, item2 }
    }
}
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
