#[clappen::clappen(export = prefixed_struct_generator, default_prefix = "")]
mod m1 {
    pub struct ServerOptions {
        pub url: String,
    }
}

fn main() {}
