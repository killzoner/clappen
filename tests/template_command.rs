// The macro way: `#[clappen_command]` flattening a child that carries a
// `#[clappen_template_impl]` auto-generates the child's `From<Prefixed> for Base` in the
// template-free parent, with no opt-in.

#[clappen::clappen(export = remote)]
mod remote {
    #[derive(Debug, PartialEq)]
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

remote!();

#[clappen::clappen(export = options)]
mod options {
    // no template of its own: just flattens the same remote under two prefixes
    #[derive(Debug)]
    pub struct Options {
        #[clappen_command(apply = remote, prefix = "test1")]
        pub nested1: Remote,
        #[clappen_command(apply = remote, prefix = "test2")]
        pub nested2: Remote,
    }
}

options!();

#[test]
fn flattened_fields_map_to_base() {
    let nested1 = __inner_nested1::Test1Remote {
        test1_id: String::from("h"),
    };
    let nested2 = __inner_nested2::Test2Remote {
        test2_id: String::from("h"),
    };

    let nested1: Remote = nested1.into();
    let nested2: Remote = nested2.into();

    assert_eq!(nested1, nested2);
    assert_eq!(
        nested1,
        Remote {
            id: String::from("h"),
        }
    );
}
