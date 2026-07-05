//! One `Remote` flattened three times in one clap parser: unprefixed, `primary` and `secondary`.
//! Its `#[clappen_template_impl]` gives each copy a `From<_> for Remote`, so the parsed values
//! compare with `==`.

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
}

nested!();

#[clappen::clappen(export = prefixed_struct_generator)]
mod prefixed_struct_generator {
    #[derive(clap::Parser)]
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

    let base: Remote = options.base.into();
    let primary: Remote = options.primary.into();
    let secondary: Remote = options.secondary.into();

    println!("base == primary?      {}", base == primary);
    println!("primary == secondary? {}", primary == secondary);
}
