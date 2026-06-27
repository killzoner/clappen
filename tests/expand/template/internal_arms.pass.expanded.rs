#![allow(dead_code)]
/// Macros used for nested struct definition : []
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
/// Template impl for `Remote` (prefix 'test1')
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
/// Template impl for `Remote` (prefix 'test2')
impl From<Test2Remote> for Remote {
    fn from(value: Test2Remote) -> Self {
        Self { id: value.test2_id }
    }
}
fn main() {}
