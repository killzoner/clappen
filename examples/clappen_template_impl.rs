//! One options struct, flattened several ways under one clap parser via `#[clappen_command]`:
//! unprefixed, plus `primary` and `secondary` prefixes. The struct carries a
//! `#[clappen_template_impl]`, so each flattened field's `From<_> for Remote` is generated
//! automatically; every parse converts back to `Remote` and compares with `==`.

use clap::Parser;

#[clappen::clappen(export = remote)]
mod remote {
    #[derive(clap::Args, Debug, PartialEq)]
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

remote!();

#[clappen::clappen(export = options)]
mod options {
    #[derive(clap::Parser, Debug)]
    pub struct Options {
        #[command(flatten)]
        #[clappen_command(apply = remote)]
        pub base: Remote,
        #[command(flatten)]
        #[clappen_command(apply = remote, prefix = "primary")]
        pub primary: Remote,
        #[command(flatten)]
        #[clappen_command(apply = remote, prefix = "secondary")]
        pub secondary: Remote,
    }
}

options!();

// cargo run --example clappen_template_impl -- --id h --primary-id h --secondary-id x
fn main() {
    let options = Options::parse();

    let base: Remote = options.base.into();
    let primary: Remote = options.primary.into();
    let secondary: Remote = options.secondary.into();

    println!("base      = {base:?}");
    println!("primary   = {primary:?}");
    println!("secondary = {secondary:?}");

    println!("base == primary?      {}", base == primary);
    println!("primary == secondary? {}", primary == secondary);
}
