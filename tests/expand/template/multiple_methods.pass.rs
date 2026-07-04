// Multiple methods in one `#[clappen_template_impl]` block: a two-method trait, written once and
// generated per prefix. `narrow` and `widen` reuse the param name `value` with different tags
// (`Prefixed` vs `Base`), so each method resolves its own binding. A trait impl cannot be split
// into separate blocks, so this shape needs per-method binding scoping.

trait Convert<T> {
    fn narrow(value: T) -> Self;
    fn widen(value: Self) -> T;
}

#[clappen::clappen(export = options)]
mod options {
    struct Options {
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
    let base = Options {
        url: String::from("x"),
    };
    let prefixed: TestOptions = Options::widen(base);
    let _roundtrip: Options = Options::narrow(prefixed);
}
