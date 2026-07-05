# Limitations

The examples use a mod exported as `server`, with the struct `ServerOptions` and its field `url`. `server!("test")` makes the prefixed copy `TestServerOptions`, with the field `test_url`.

## `#[clappen]`

- Providing custom `long` or `env` to your fields is not supported.

  ```rust
  struct ServerOptions {
      #[arg(long = "address")] // every copy gets the same `--address`
      url: String,
  }
  ```

- Probably some edge cases are not covered. For example, `clap` `subcommand`s should be working, but it was not tested extensively as my use case is mainly reusing `struct`s in a mono repository fashion.

- If you invoke the generated macro twice in one module (`server!()` and `server!("test")`), a `use` item in the mod gives the error E0252. Write full paths, or put the `use` item outside the mod.

  ```rust
  // fails
  use std::fmt;
  impl fmt::Display for ServerOptions { /* .. */ }

  // works
  impl std::fmt::Display for ServerOptions { /* .. */ }
  ```

- A `macro_rules!` in the mod gives a compile error when it uses a repetition (`$(..)*`) or a metavariable that the generated macro also uses (`$prefix`, `$field`, `$command_prefix`, `$parent_default`). Define the macro outside the mod, before it.

  ```rust
  // works: the macro before the mod
  macro_rules! sum {
      ($($x:expr),*) => { 0 $(+ $x)* };
  }

  #[clappen::clappen(export = server)]
  mod server {
      struct ServerOptions {
          url: String,
      }

      impl ServerOptions {
          fn total(&self) -> u32 {
              // fails: the same `macro_rules! sum` here
              sum!(1, 2)
          }
      }
  }
  ```

## `impl` blocks in the mod

- References to fields with more than 1 nesting level won't work - like `self.my_field_level1.my_field_level2`.\
  \
  Generally this should not be needed because you want to get something *out* of your reusable struct parser (and Rust might get in your way for borrow-related things).\
  \
  If you *still* want to do that, you can write custom getter functions, and the renamed fields will be picked in the `impl` blocks.

  ```rust
  // fails: `nested` is a flattened child
  fn nested_url(&self) -> String { self.nested.url.clone() }

  // works: with `fn url(&self) -> String` in the child's own impl
  fn nested_url(&self) -> String { self.nested.url() }
  ```

## `#[clappen_template_impl]`

In the copy for `"test"`, this template reads `value.test_url`:

```rust
#[clappen_template_impl]
impl From<Prefixed> for Base {
    fn from(value: Prefixed) -> Self {
        Self { url: value.url }
    }
}
```

A macro cannot see types. Thus clappen renames a prefixed field only when it reads the field on one of these values:

- `self`, and `Self { .. }` in an impl for a tag.
- A method param, a `let` or a closure param with a tag type: `value: Prefixed`, `value: &Prefixed`, `other: &Self`, `value: Box<Prefixed>` (also `Rc` and `Arc`).
- An element of a typed tuple or array pattern: `(value, n): (Prefixed, u8)`.

The code in the table that follows gives a compile error in the prefixed copies.

| Code | Error | Workaround |
|---|---|---|
| A field read on another expression: `values[0].url`, `get().url`, `(*value).url` | E0609 | Give the value a name with a type: `let one: &Prefixed = &values[0];`, then `one.url`. Write `value.url` in place of `(*value).url` |
| A `fn` or an `impl` in a method body: `fn url_of(v: &Prefixed) -> String { v.url.clone() }` | E0609 or E0560 | Move it out of the method body, for example into the template impl |
| A value with no tag type: `let copy = value.clone();`, `\|x\| x.url`, `for v in &values`, `if let Some(v) = opt`, `Option<Prefixed>` | E0609 | Give the value a tag type: `\|x: &Prefixed\| x.url`, `let v: &Prefixed = v;` |
| A tag-typed name used again for a value of another type in the same method: `let value = Fallback::default();`, `Some(value) =>` | E0609 | Use another name |
| A field or a tag in a macro body that is not a list of expressions or statements: `info!(url = %self.url)`, `json!({"url": self.url})`, `matches!(v, Some(x) if x == self.url)`, `my_macro!(key => Self { url })` | E0609, E0560, E0425 or E0433 | Make or read the value before the macro: `let url = &self.url;`, then `info!(url = %url)` |
| A field or a tag in the args of an attribute: `#[instrument(fields(url = %self.url))]`, `#[my_attr(Prefixed)]` | E0609 or E0425, in the code that the attribute makes | Read the field in the method body |
| A local alias of a tag: `type P = Prefixed;`, `use Prefixed as P;` | E0609 or E0432 | Write the tag |
| A field read through a flattened child: `self.nested.url` | E0609 | Add a getter in a regular impl of the child (`self.nested.url()`), or convert the child first (`MyStruct::from(value.nested).url`) |
| An impl with no `Prefixed` in its header: `impl Base { .. }`, `impl Display for Base` | E0592 or E0119, with two or more copies | Put `Prefixed` in the header (`impl From<Prefixed> for Base`). If the impl does not use `Prefixed`, write a regular impl |
| A template method that one copy does not use, with `-D warnings` | `dead_code` | Add `#[allow(dead_code)]` to the template impl |

Exception: a name keeps its tag up to the end of the method. In the copy for `"test"`:

```rust
fn from(value: Prefixed) -> Self {
    let value = Fallback::default();
    let url = value.url; // becomes `value.test_url`
    // ..
}
```

If `Fallback` has a field `test_url`, this compiles and reads that field. A `fn` in the method body has the same problem: in `fn check(value: &Fallback)`, `value.url` also becomes `value.test_url`.
