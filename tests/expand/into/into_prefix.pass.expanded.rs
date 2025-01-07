/// Macros used for nested struct definition : []
/// Struct with prefix '', default_prefix: 'default'
struct DefaultSimpleStruct {
    default_item1: u64,
    default_item2: String,
}
/// Macros used for nested struct definition : []
/// Struct with prefix 'one', default_prefix: 'default'
struct OneDefaultSimpleStruct {
    one_default_item1: u64,
    one_default_item2: String,
}
/// Fields with prefix: [item1,item2]
#[allow(clippy::from_over_into)]
impl Into<DefaultSimpleStruct> for OneDefaultSimpleStruct {
    fn into(self) -> DefaultSimpleStruct {
        let default_item1 = self.one_default_item1.into();
        let default_item2 = self.one_default_item2.into();
        DefaultSimpleStruct {
            default_item1,
            default_item2,
        }
    }
}
/// Macros used for nested struct definition : []
/// Struct with prefix 'two', default_prefix: 'default'
struct TwoDefaultSimpleStruct {
    two_default_item1: u64,
    two_default_item2: String,
}
/// Fields with prefix: [item1,item2]
#[allow(clippy::from_over_into)]
impl Into<DefaultSimpleStruct> for TwoDefaultSimpleStruct {
    fn into(self) -> DefaultSimpleStruct {
        let default_item1 = self.two_default_item1.into();
        let default_item2 = self.two_default_item2.into();
        DefaultSimpleStruct {
            default_item1,
            default_item2,
        }
    }
}
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
