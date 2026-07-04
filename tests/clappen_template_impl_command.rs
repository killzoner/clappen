// A parent without a template still gets its flattened children's `From` impls.

#[clappen::clappen(export = nested)]
mod nested {
    #[derive(Debug, PartialEq)]
    pub struct MyStruct {
        pub id: String,
    }

    #[clappen_template_impl]
    impl From<Prefixed> for Base {
        fn from(value: Prefixed) -> Self {
            Self { id: value.id }
        }
    }
}

nested!();

#[clappen::clappen(export = options)]
mod options {
    #[derive(Debug)]
    pub struct ServerOptions {
        #[clappen_command(apply = nested, prefix = "test1")]
        pub nested1: MyStruct,
        #[clappen_command(apply = nested, prefix = "test2")]
        pub nested2: MyStruct,
    }
}

options!();

#[test]
fn flattened_fields_map_to_base() {
    let nested1 = __inner_nested1::Test1MyStruct {
        test1_id: String::from("h"),
    };
    let nested2 = __inner_nested2::Test2MyStruct {
        test2_id: String::from("h"),
    };

    let nested1: MyStruct = nested1.into();
    let nested2: MyStruct = nested2.into();

    assert_eq!(nested1, nested2);
    assert_eq!(
        nested1,
        MyStruct {
            id: String::from("h"),
        }
    );
}
