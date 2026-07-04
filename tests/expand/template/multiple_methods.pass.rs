// A two-method trait impl, which cannot be split: `value` is `Prefixed` in `narrow` and `Base` in
// `widen`, so each method resolves its own binding.

trait Convert<T> {
    fn narrow(value: T) -> Self;
    fn widen(value: Self) -> T;
}

#[clappen::clappen(export = options)]
mod options {
    struct ServerOptions {
        url: String,
    }

    #[clappen_template_impl]
    impl Convert<Prefixed> for Base {
        fn narrow(value: Prefixed) -> Base {
            Base { url: value.url }
        }
        fn widen(value: Base) -> Prefixed {
            Prefixed { url: value.url }
        }
    }
}

options!();
options!("test");

fn main() {
    let base = ServerOptions {
        url: String::from("x"),
    };
    let prefixed: TestServerOptions = ServerOptions::widen(base);
    let _roundtrip: ServerOptions = ServerOptions::narrow(prefixed);
}
