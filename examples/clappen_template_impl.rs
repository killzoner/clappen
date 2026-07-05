//! One `Remote` flattened three times in one clap parser: unprefixed, `primary` and `secondary`.
//! Its two `#[clappen_template_impl]` blocks convert each copy to `Remote` and back, so the parsed
//! values compare with `==`.
//!
//! For nested structs, see `from_nested` and `from_deep_nested` in
//! `tests/clappen_template_impl_from.rs`: a parent with its own template converts each child in
//! two steps, so the child needs both templates.

use clap::Parser;

#[clappen::clappen(export = nested)]
mod nested {
    #[derive(clap::Args, PartialEq)]
    pub struct Remote {
        #[arg(long)]
        pub id: String,
    }

    // written once, generated as a `From<Prefixed>` for each flattened variant
    #[clappen_template_impl]
    impl From<Prefixed> for Base {
        fn from(value: Prefixed) -> Self {
            Self { id: value.id }
        }
    }

    // the reverse, written once too: a `Remote` converts into each flattened variant
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

    // the reverse template: `into()` builds each flattened copy from a plain `Remote`
    let expected = ServerOptions {
        base: Remote::new("h").into(),
        primary: Remote::new("h").into(),
        secondary: Remote::new("x").into(),
    };
    println!("options == expected?  {}", options == expected);

    let base: Remote = options.base.into();
    let primary: Remote = options.primary.into();
    let secondary: Remote = options.secondary.into();

    println!("base == primary?      {}", base == primary);
    println!("primary == secondary? {}", primary == secondary);
}
