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
}

fn main() {
    plain!();
    plain!("first");
    generic!();
    generic!("first");
}
