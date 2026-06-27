#[allow(dead_code, clippy::missing_safety_doc)]
mod tests {

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

    #[clappen::clappen(export = plain)]
    mod plain {

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
    }
    plain!();
    plain!("first");

    #[clappen::clappen(export = generic)]
    mod generic {

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

        impl<A, I> GenTrait<SpecificType> for GenericStruct<I>
        where
            A: std::fmt::Display,
            I: std::iter::Iterator<Item = A> + Clone,
        {
            fn baz(&self) -> String {
                self.iter
                    .clone()
                    .flat_map(|s| s.to_string().chars().collect::<Vec<_>>().into_iter())
                    .collect()
            }
        }

        unsafe impl<A, I> UnsafeTrait for GenericStruct<I>
        where
            A: std::fmt::Display,
            I: std::iter::Iterator<Item = A> + Clone,
        {
            fn bar(&self) -> String {
                self.iter
                    .clone()
                    .flat_map(|s| s.to_string().chars().collect::<Vec<_>>().into_iter())
                    .collect()
            }
        }
    }
    generic!();
    generic!("first");

    #[test]
    fn plain() {
        let a = Struct {};
        let b = FirstStruct {};
        assert_eq!(a.foo(), b.foo());
        assert_eq!(a.bar(), b.bar());
        assert_eq!(a.baz(), b.baz());
    }

    #[test]
    fn generic() {
        let s_vec = ["Howdy", " ", "There"];
        let a = GenericStruct { iter: s_vec.iter() };
        let b = FirstGenericStruct {
            first_iter: s_vec.iter(),
        };
        assert_eq!(a.foo(), b.foo());
        assert_eq!(a.bar(), b.bar());
        assert_eq!(a.baz(), b.baz());
    }
}
