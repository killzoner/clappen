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
- A copy of one of these values: `let other = value;`, `&value` or `value.clone()`.

The code in the table that follows gives a compile error in the prefixed copies.

| Code | Error | Workaround |
|---|---|---|
| A field read on another expression: `values[0].url`, `get().url` | E0609 | Give the value a name with a type: `let one: &Prefixed = &values[0];`, then `one.url` |
| A value with no tag type: `\|x\| x.url`, `for v in &values`, `if let Some(v) = opt`, `Option<Prefixed>` | E0609 | Give the value a tag type: `\|x: &Prefixed\| x.url`, `let v: &Prefixed = v;` |
| A tag-typed name used again for a value of another type in the same method: `let value = Fallback::default();`, `Some(value) =>` | E0609 | Use another name |
| A struct literal in a macro body that is not a list of expressions or statements: `my_macro!(key => Self { url })` | E0560 | Make the value before the macro: `let options = Self { url };` |
| A local alias of a tag: `type P = Prefixed;`, `use Prefixed as P;` | E0609 or E0432 | Write the tag |
| An associated type set to `Self`: `type Output = Self;`, then `Self::Output { url }` or `value: Self::Output` | E0560 or E0609 | Write the tag: `type Output = Prefixed;` |
| A field read through a flattened child: `self.nested.url` | E0609 | Add a getter in a regular impl of the child (`self.nested.url()`), or convert the child first (`MyStruct::from(value.nested).url`) |
| An impl with no `Prefixed` in its header: `impl Base { .. }`, `impl Display for Base` | E0592 or E0119, with two or more copies | Put `Prefixed` in the header (`impl From<Prefixed> for Base`). If the impl does not use `Prefixed`, write a regular impl |
| A template method that one copy does not use, with `-D warnings` | `dead_code` | Add `#[allow(dead_code)]` to the template impl |
