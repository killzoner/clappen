#[clappen::clappen(export = remote)]
mod remote {
    pub struct Remote {
        pub id: String,
    }

    #[clappen_template_impl]
    impl From<Prefixed> for Base {
        fn from(value: Prefixed) -> Self {
            Self { id: value.id }
        }
    }
}

#[clappen::clappen(export = prefixed_struct_generator)]
mod prefixed_struct_generator {
    pub struct ServerOptions {
        pub url: String,
        #[clappen_command(apply = remote, prefix = "test")]
        pub remote: Remote,
    }

    // the nested field's sibling conversion is generated at the outer scope
    #[clappen_template_impl]
    impl From<Prefixed> for Base {
        fn from(value: Prefixed) -> Self {
            Self {
                url: value.url,
                remote: value.remote.into(),
            }
        }
    }
}

prefixed_struct_generator!();
prefixed_struct_generator!("test1");

fn main() {
    let value = Test1ServerOptions {
        test1_url: String::from("hi"),
        test1_remote: __inner_test1_remote::Test1TestRemote {
            test1_test_id: String::from("x"),
        },
    };
    let _: ServerOptions = value.into();
}
