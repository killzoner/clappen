#![allow(dead_code)]
struct MyStruct {}

struct ChildMyStruct{}

struct ChildChildMyStruct{}

clappen::__into_impl!(fields = [], prefixes = [child, child, MyStruct]);
