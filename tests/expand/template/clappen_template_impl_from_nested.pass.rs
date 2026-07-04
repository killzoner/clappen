#[clappen::clappen(export = nested)]
mod nested {
    pub struct MyStruct {
        pub id: String,
    }

    #[clappen_template_impl]
    impl From<Prefixed> for Base {
        fn from(value: Prefixed) -> Self {
            Self { id: value.id }
        }
    }

    #[clappen_template_impl]
    impl From<Base> for Prefixed {
        fn from(value: Base) -> Self {
            Self { id: value.id }
        }
    }
}

#[clappen::clappen(export = prefixed_struct_generator)]
mod prefixed_struct_generator {
    pub struct ServerOptions {
        pub url: String,
        #[clappen_command(apply = nested, prefix = "test")]
        pub nested: MyStruct,
    }

    // the child's conversion is generated next to the parent's
    #[clappen_template_impl]
    impl From<Prefixed> for Base {
        fn from(value: Prefixed) -> Self {
            Self {
                url: value.url,
                nested: MyStruct::from(value.nested).into(),
            }
        }
    }
}

nested!();
prefixed_struct_generator!();
prefixed_struct_generator!("test1");

fn main() {
    let value = Test1ServerOptions {
        test1_url: String::from("hi"),
        test1_nested: __inner_test1_nested::Test1TestMyStruct {
            test1_test_id: String::from("x"),
        },
    };
    let _: ServerOptions = value.into();
}
