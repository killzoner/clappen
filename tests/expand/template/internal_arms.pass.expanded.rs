#![allow(dead_code)]
/// Macros used for nested struct definition : []
/// Struct with prefix '', default_prefix: ''
pub struct Remote {
    pub id: String,
}
/// Macros used for nested struct definition : []
/// Struct with prefix 'test', default_prefix: ''
pub struct TestRemote {
    pub test_id: String,
}
/// Macros used for nested struct definition : []
/// Struct with prefix 'test1', default_prefix: ''
pub struct Test1Remote {
    pub test1_id: String,
}
/// Template impl for `Remote` (prefix 'test1', nested via [])
impl From<Test1Remote> for Remote {
    fn from(value: Test1Remote) -> Self {
        Self { id: value.test1_id }
    }
}
/// Macros used for nested struct definition : []
/// Struct with prefix 'test2', default_prefix: ''
pub struct Test2Remote {
    pub test2_id: String,
}
/// Template impl for `Remote` (prefix 'test2', nested via [])
impl From<Test2Remote> for Remote {
    fn from(value: Test2Remote) -> Self {
        Self { id: value.test2_id }
    }
}
mod __inner_nested {
    /// Macros used for nested struct definition : []
    /// Struct with prefix '', default_prefix: ''
    pub struct Remote {
        pub id: String,
    }
    /// Macros used for nested struct definition : []
    /// Struct with prefix 'Bp', default_prefix: ''
    pub struct BpRemote {
        pub bp_id: String,
    }
}
mod __inner_pd_nested {
    /// Macros used for nested struct definition : []
    /// Struct with prefix 'Pd', default_prefix: ''
    pub struct PdRemote {
        pub pd_id: String,
    }
}
mod __inner_pd4_nested {
    /// Macros used for nested struct definition : []
    /// Struct with prefix 'Pd4Dp', default_prefix: ''
    pub struct Pd4DpRemote {
        pub pd4_dp_id: String,
    }
}
mod __inner_c4_pd4_nested {
    /// Macros used for nested struct definition : []
    /// Struct with prefix 'C4Pd4Dp', default_prefix: ''
    pub struct C4Pd4DpRemote {
        pub c4_pd4_dp_id: String,
    }
}
/// Template impl for `Remote` (prefix '', nested via [])
impl From<__inner_nested::Remote> for Remote {
    fn from(value: __inner_nested::Remote) -> Self {
        Self { id: value.id }
    }
}
/// Template impl for `Remote` (prefix '', nested via [bp])
impl From<__inner_nested::BpRemote> for Remote {
    fn from(value: __inner_nested::BpRemote) -> Self {
        Self { id: value.bp_id }
    }
}
/// Template impl for `Remote` (prefix '', nested via [])
impl From<__inner_pd_nested::PdRemote> for Remote {
    fn from(value: __inner_pd_nested::PdRemote) -> Self {
        Self { id: value.pd_id }
    }
}
/// Template impl for `Remote` (prefix '', nested via [dp])
impl From<__inner_pd4_nested::Pd4DpRemote> for Remote {
    fn from(value: __inner_pd4_nested::Pd4DpRemote) -> Self {
        Self { id: value.pd4_dp_id }
    }
}
/// Template impl for `Remote` (prefix 'c4', nested via [dp])
impl From<__inner_c4_pd4_nested::C4Pd4DpRemote> for __inner_pd4_nested::Pd4DpRemote {
    fn from(value: __inner_c4_pd4_nested::C4Pd4DpRemote) -> Self {
        Self {
            pd4_dp_id: value.c4_pd4_dp_id,
        }
    }
}
fn main() {}
