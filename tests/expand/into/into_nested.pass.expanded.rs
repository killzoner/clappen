/// Macros used for nested struct definition : []
pub struct NestedStruct {
    item1: u64,
    item2: String,
}
mod __inner_default_item1 {
    /// Macros used for nested struct definition : []
    /// Struct with prefix 'DefaultTest1', default_prefix: ''
    pub struct DefaultTest1NestedStruct {
        default_test1_item1: u64,
        default_test1_item2: String,
    }
    /// Fields with prefix: [item1,item2]
    #[allow(clippy::from_over_into)]
    impl Into<NestedStruct> for DefaultTest1NestedStruct {
        fn into(self) -> NestedStruct {
            let item1 = self.default_test1_item1.into();
            let item2 = self.default_test1_item2.into();
            NestedStruct { item1, item2 }
        }
    }
}
mod __inner_default_item2 {
    /// Macros used for nested struct definition : []
    /// Struct with prefix 'DefaultTest2', default_prefix: ''
    pub struct DefaultTest2NestedStruct {
        default_test2_item1: u64,
        default_test2_item2: String,
    }
    /// Fields with prefix: [item1,item2]
    #[allow(clippy::from_over_into)]
    impl Into<NestedStruct> for DefaultTest2NestedStruct {
        fn into(self) -> NestedStruct {
            let item1 = self.default_test2_item1.into();
            let item2 = self.default_test2_item2.into();
            NestedStruct { item1, item2 }
        }
    }
}
/// Macros used for nested struct definition : [nested,nested]
/// Struct with prefix '', default_prefix: 'default'
pub struct DefaultSimpleStruct {
    default_item1: __inner_default_item1::DefaultTest1NestedStruct,
    default_item2: __inner_default_item2::DefaultTest2NestedStruct,
}
mod __inner_one_default_item1 {
    /// Macros used for nested struct definition : []
    /// Struct with prefix 'OneDefaultTest1', default_prefix: ''
    pub struct OneDefaultTest1NestedStruct {
        one_default_test1_item1: u64,
        one_default_test1_item2: String,
    }
    /// Fields with prefix: [item1,item2]
    #[allow(clippy::from_over_into)]
    impl Into<NestedStruct> for OneDefaultTest1NestedStruct {
        fn into(self) -> NestedStruct {
            let item1 = self.one_default_test1_item1.into();
            let item2 = self.one_default_test1_item2.into();
            NestedStruct { item1, item2 }
        }
    }
}
mod __inner_one_default_item2 {
    /// Macros used for nested struct definition : []
    /// Struct with prefix 'OneDefaultTest2', default_prefix: ''
    pub struct OneDefaultTest2NestedStruct {
        one_default_test2_item1: u64,
        one_default_test2_item2: String,
    }
    /// Fields with prefix: [item1,item2]
    #[allow(clippy::from_over_into)]
    impl Into<NestedStruct> for OneDefaultTest2NestedStruct {
        fn into(self) -> NestedStruct {
            let item1 = self.one_default_test2_item1.into();
            let item2 = self.one_default_test2_item2.into();
            NestedStruct { item1, item2 }
        }
    }
}
/// Macros used for nested struct definition : [nested,nested]
/// Struct with prefix 'one', default_prefix: 'default'
pub struct OneDefaultSimpleStruct {
    one_default_item1: __inner_one_default_item1::OneDefaultTest1NestedStruct,
    one_default_item2: __inner_one_default_item2::OneDefaultTest2NestedStruct,
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
mod __inner_two_default_item1 {
    /// Macros used for nested struct definition : []
    /// Struct with prefix 'TwoDefaultTest1', default_prefix: ''
    pub struct TwoDefaultTest1NestedStruct {
        two_default_test1_item1: u64,
        two_default_test1_item2: String,
    }
    /// Fields with prefix: [item1,item2]
    #[allow(clippy::from_over_into)]
    impl Into<NestedStruct> for TwoDefaultTest1NestedStruct {
        fn into(self) -> NestedStruct {
            let item1 = self.two_default_test1_item1.into();
            let item2 = self.two_default_test1_item2.into();
            NestedStruct { item1, item2 }
        }
    }
}
mod __inner_two_default_item2 {
    /// Macros used for nested struct definition : []
    /// Struct with prefix 'TwoDefaultTest2', default_prefix: ''
    pub struct TwoDefaultTest2NestedStruct {
        two_default_test2_item1: u64,
        two_default_test2_item2: String,
    }
    /// Fields with prefix: [item1,item2]
    #[allow(clippy::from_over_into)]
    impl Into<NestedStruct> for TwoDefaultTest2NestedStruct {
        fn into(self) -> NestedStruct {
            let item1 = self.two_default_test2_item1.into();
            let item2 = self.two_default_test2_item2.into();
            NestedStruct { item1, item2 }
        }
    }
}
/// Macros used for nested struct definition : [nested,nested]
/// Struct with prefix 'two', default_prefix: 'default'
pub struct TwoDefaultSimpleStruct {
    two_default_item1: __inner_two_default_item1::TwoDefaultTest1NestedStruct,
    two_default_item2: __inner_two_default_item2::TwoDefaultTest2NestedStruct,
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
fn main() {}
