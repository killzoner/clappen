#![allow(dead_code, clippy::missing_safety_doc)]
type SpecificType = ();
trait Trait {
    fn foo(&self) -> String;
}
unsafe trait UnsafeTrait {
    fn bar(&self) -> String;
}
trait GenTrait<A> {
    fn baz(&self) -> String;
}
fn main() {
    /// Macros used for nested struct definition : []
    struct Struct {}
    impl Trait for Struct {
        fn foo(&self) -> String {
            String::from("Struct Trait Impl")
        }
    }
    impl GenTrait<SpecificType> for Struct {
        fn baz(&self) -> String {
            String::from("Struct GenTrait Impl")
        }
    }
    unsafe impl UnsafeTrait for Struct {
        fn bar(&self) -> String {
            String::from("Struct UnsafeTrait Impl")
        }
    }
    /// Macros used for nested struct definition : []
    /// Struct with prefix 'first', default_prefix: ''
    struct FirstStruct {}
    /// Fields with prefix: []
    impl Trait for FirstStruct {
        fn foo(&self) -> String {
            String::from("Struct Trait Impl")
        }
    }
    /// Fields with prefix: []
    impl GenTrait<SpecificType> for FirstStruct {
        fn baz(&self) -> String {
            String::from("Struct GenTrait Impl")
        }
    }
    /// Fields with prefix: []
    unsafe impl UnsafeTrait for FirstStruct {
        fn bar(&self) -> String {
            String::from("Struct UnsafeTrait Impl")
        }
    }
    /// Macros used for nested struct definition : []
    struct GenericStruct<A> {
        iter: A,
    }
    impl<A, I> Trait for GenericStruct<I>
    where
        A: std::fmt::Display,
        I: std::iter::Iterator<Item = A> + Clone,
    {
        fn foo(&self) -> String {
            self.iter
                .clone()
                .flat_map(|s| s.to_string().chars().collect::<Vec<_>>().into_iter())
                .collect()
        }
    }
    /// Macros used for nested struct definition : []
    /// Struct with prefix 'first', default_prefix: ''
    struct FirstGenericStruct<A> {
        first_iter: A,
    }
    /// Fields with prefix: [iter]
    impl<A, I> Trait for FirstGenericStruct<I>
    where
        A: std::fmt::Display,
        I: std::iter::Iterator<Item = A> + Clone,
    {
        fn foo(&self) -> String {
            self.first_iter
                .clone()
                .flat_map(|s| s.to_string().chars().collect::<Vec<_>>().into_iter())
                .collect()
        }
    }
}
