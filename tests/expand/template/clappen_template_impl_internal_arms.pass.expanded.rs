#![allow(dead_code)]
/// Macros used for nested struct definition : []
/// Struct with prefix '', default_prefix: ''
pub struct MyStruct {
    pub id: String,
}
/// Macros used for nested struct definition : []
/// Struct with prefix 'test', default_prefix: ''
pub struct TestMyStruct {
    pub test_id: String,
}
/// Macros used for nested struct definition : []
/// Struct with prefix 'test1', default_prefix: ''
pub struct Test1MyStruct {
    pub test1_id: String,
}
/// Template impl for `MyStruct` (prefix 'test1', nested via [])
impl From<Test1MyStruct> for MyStruct {
    fn from(value: Test1MyStruct) -> Self {
        Self { id: value.test1_id }
    }
}
/// Macros used for nested struct definition : []
/// Struct with prefix 'test2', default_prefix: ''
pub struct Test2MyStruct {
    pub test2_id: String,
}
/// Template impl for `MyStruct` (prefix 'test2', nested via [])
impl From<Test2MyStruct> for MyStruct {
    fn from(value: Test2MyStruct) -> Self {
        Self { id: value.test2_id }
    }
}
mod __inner_nested {
    /// Macros used for nested struct definition : []
    /// Struct with prefix '', default_prefix: ''
    pub struct MyStruct {
        pub id: String,
    }
    /// Macros used for nested struct definition : []
    /// Struct with prefix 'Bp', default_prefix: ''
    pub struct BpMyStruct {
        pub bp_id: String,
    }
}
mod __inner_pd_nested {
    /// Macros used for nested struct definition : []
    /// Struct with prefix 'Pd', default_prefix: ''
    pub struct PdMyStruct {
        pub pd_id: String,
    }
}
mod __inner_pd4_nested {
    /// Macros used for nested struct definition : []
    /// Struct with prefix 'Pd4Dp', default_prefix: ''
    pub struct Pd4DpMyStruct {
        pub pd4_dp_id: String,
    }
}
mod __inner_c4_pd4_nested {
    /// Macros used for nested struct definition : []
    /// Struct with prefix 'C4Pd4Dp', default_prefix: ''
    pub struct C4Pd4DpMyStruct {
        pub c4_pd4_dp_id: String,
    }
}
/// Template impl for `MyStruct` (prefix '', nested via [])
impl From<__inner_nested::MyStruct> for MyStruct {
    fn from(value: __inner_nested::MyStruct) -> Self {
        Self { id: value.id }
    }
}
/// Template impl for `MyStruct` (prefix '', nested via [bp])
impl From<__inner_nested::BpMyStruct> for MyStruct {
    fn from(value: __inner_nested::BpMyStruct) -> Self {
        Self { id: value.bp_id }
    }
}
/// Template impl for `MyStruct` (prefix '', nested via [])
impl From<__inner_pd_nested::PdMyStruct> for MyStruct {
    fn from(value: __inner_pd_nested::PdMyStruct) -> Self {
        Self { id: value.pd_id }
    }
}
/// Template impl for `MyStruct` (prefix '', nested via [dp])
impl From<__inner_pd4_nested::Pd4DpMyStruct> for MyStruct {
    fn from(value: __inner_pd4_nested::Pd4DpMyStruct) -> Self {
        Self { id: value.pd4_dp_id }
    }
}
/// Template impl for `MyStruct` (prefix 'c4', nested via [dp])
impl From<__inner_c4_pd4_nested::C4Pd4DpMyStruct> for __inner_pd4_nested::Pd4DpMyStruct {
    fn from(value: __inner_c4_pd4_nested::C4Pd4DpMyStruct) -> Self {
        Self {
            pd4_dp_id: value.c4_pd4_dp_id,
        }
    }
}
fn main() {}
