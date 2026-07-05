use clap::Parser;

#[clappen::clappen(export = nested)]
mod nested {
    #[derive(clap::Args, PartialEq)]
    pub struct Remote {
        #[arg(long)]
        pub id: String,
    }

    // generates the `From<Prefixed>` impl for each flattened variant
    #[clappen_template_impl]
    impl From<Prefixed> for Base {
        fn from(value: Prefixed) -> Self {
            Self { id: value.id }
        }
    }

    // generates the `From<Base>` impl for each flattened variant
    #[clappen_template_impl]
    impl From<Base> for Prefixed {
        fn from(value: Base) -> Self {
            Self { id: value.id }
        }
    }
}

nested!();

impl Remote {
    fn new(id: &str) -> Self {
        Self {
            id: String::from(id),
        }
    }
}

#[clappen::clappen(export = prefixed_struct_generator)]
mod prefixed_struct_generator {
    #[derive(clap::Parser, PartialEq)]
    pub struct ServerOptions {
        #[command(flatten)]
        #[clappen_command(apply = nested)]
        pub base: Remote,
        #[command(flatten)]
        #[clappen_command(apply = nested, prefix = "primary")]
        pub primary: Remote,
        #[command(flatten)]
        #[clappen_command(apply = nested, prefix = "secondary")]
        pub secondary: Remote,
    }
}

prefixed_struct_generator!();

// cargo run --example clappen_template_impl -- --id h --primary-id h --secondary-id x
fn main() {
    let options = ServerOptions::parse();

    // uses the generated `From<Base>` impls
    let expected = ServerOptions {
        base: Remote::new("h").into(),
        primary: Remote::new("h").into(),
        secondary: Remote::new("x").into(),
    };
    println!("options == expected?  {}", options == expected);

    // uses the generated `From<Prefixed>` impls
    let base: Remote = options.base.into();
    let primary: Remote = options.primary.into();
    let secondary: Remote = options.secondary.into();

    println!("base == primary?      {}", base == primary);
    println!("primary == secondary? {}", primary == secondary);
}
