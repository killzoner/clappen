# How clappen works

`#[clappen]` does not expand to code directly. Applied to a `mod`, it generates a
`macro_rules! NAME`, and you invoke that macro (`NAME!()`, `NAME!("primary")`, ...) to generate a
prefixed copy of the struct and its impls. Three hidden proc-macros do the per-item work:
`__clappen_struct`, `__clappen_impl`, `__clappen_template_impl`.

## 1. The macro arms

Each `NAME!` invocation matches one arm:

- `NAME!()` (base): the base struct, its regular impls, and the template impls of its flattened children, for the copies that the base struct holds.
- `NAME!("p")` (prefixed): a prefixed copy of the struct, its regular impls, its template impls, and the template impls of its flattened children.
- `NAME!(@__struct "p")`: the same as `NAME!("p")` without the template impls, for a nested field's type. The prefix is optional: a field flattened with no prefix, in a parent with no prefix, calls `NAME!(@__struct)`.
- `NAME!(@__template "p", chain = [ .. ])`: this struct's template impls for the nested copy at the path in `chain` (called by the parent that flattens this struct).

`("p")` and `@__template` build the same thing (this struct's template impls, then recursion into
its flattened fields), differing only in the chain: `("p")` starts it empty, `@__template` continues
a parent's. So the natural way to write `("p")` would be to call `@__template` on itself:

```rust
($prefix: literal) => {
    // ... prefixed struct ...
    NAME!(@__template $prefix, chain = []);   // does NOT work cross-crate
};
```

That self-call does not work as written. Neither plain way to name the macro works in both
same-crate and cross-crate use:

- **bare `NAME!`** resolves at the call site: fine when the macro is used in its own crate, but from
  crate B, `crate_a::NAME!("p")` expands a bare `NAME!` that is not in scope there, so it errors with
  `cannot find macro NAME`.
- **`$crate::NAME!`** would resolve cross-crate, but Rust rejects it in the macro's own crate: `NAME`
  is itself produced by a macro expansion (the `#[clappen]` proc-macro), and a macro-expanded
  `#[macro_export]` macro of the current crate cannot be named by an absolute path. This is the
  `macro_expanded_macro_exports_accessed_by_absolute_paths` lint, tracked at
  [rust-lang/rust#52234](https://github.com/rust-lang/rust/issues/52234). Its message is
  ``error: macro-expanded `macro_export` macros from the current crate cannot be referred to by absolute paths``.

A `pub use NAME as ALIAS;` next to the macro would make `$crate::ALIAS!` resolve in both cases, but it
forces every `#[clappen]` mod to sit at the crate root, so clappen does not use one.

So `("p")` **inlines the body**: instead of the self-call, it writes out the two pieces that
`@__template` would produce, with an empty chain:

```rust
($prefix: literal) => {
    // ... prefixed struct ...
    #[clappen::__clappen_template_impl(prefix = $prefix, chain = [], ...)]
    impl From<Prefixed> for Base { /* ... */ }                 // this struct's template impls
    CHILD!(@__template $prefix, chain = [ /* first step */ ]);  // recurse into each flattened field
};
```

Neither piece is a self-reference:

- The **first** is a proc-macro attribute, not a macro call, so there is nothing to resolve.
- The **second** does call another generated macro, but through the path the user wrote in
  `#[clappen_command(apply = ...)]` (`CHILD` above): a bare name for same-crate use, or `$crate::child`
  (or a full path) for cross-crate. The user picks a path that resolves in their crate, so clappen
  never names that child by a fixed path.

So clappen avoids naming itself. `@__template` still exists, for a parent to call when it recurses into this struct.

## 2. Nesting and the chain

`#[clappen_command(apply = CHILD, prefix = p)]` on a field flattens `CHILD` under prefix `p`. The
same field makes two calls into `CHILD`'s macro:

- `CHILD!(@__struct ...)` builds the nested struct, in a `pub(crate) mod __inner_field`.
- `CHILD!(@__template prefix, chain + step)` generates the child's template impls.

The **chain** is the list of steps that records where a nested struct sits: one `ChainStep`
`(command_prefix, field, parent_default)` per flatten level, collected from the top struct down to
the nested one. `ResolvedTag::new` combines those steps into the nested type's module path and field
prefix, which turns the `Prefixed` tag into a concrete type. The `Base` tag ignores the chain: it is
always the root type. An empty chain is a top-level instantiation, and a non-empty chain is a nested
one. The macros are defined once per `mod`. Nesting is those macros calling each other, and the
chain tells each call its position.

For example, `App` flattens `Db`, which in turn flattens `Pool` (each is a `#[clappen]` mod with its
own `#[clappen_template_impl]`, as in section 3):

```rust
struct App {
    #[clappen_command(apply = db, prefix = "db")]      // flattens Db as `database`
    database: Db,
    // ...
}
struct Db {
    #[clappen_command(apply = pool, prefix = "pool")]  // flattens Pool as `conn`
    conn: Pool,
    // ...
}
```

In `app!("x")`, `Pool` is two levels down, so `Pool`'s `@__template` call gets a two-step
chain, one step per flatten level, the top struct's step first:

```rust
pool!(@__template "x", chain = [
    // step = (command_prefix, field, parent_default). Both prefixes are optional, and
    // parent_default is left out here because neither App nor Db sets a default_prefix
    ("db",   database),   // App flattens Db   as `database`, prefix "db"
    ("pool", conn),       // Db  flattens Pool as `conn`,     prefix "pool"
]);
```

`ResolvedTag::new` combines those two steps to place `Pool`'s prefixed type at its nested module path
(each step adds one `__inner_*` level and one prefix segment). A top-level `app!("x")` starts with an
empty chain, and each flatten level appends one more step on the way down.

## 3. End-to-end walkthrough

A leaf struct with two templates, and a parent that flattens it, has a regular impl, and also has a
template:

```rust
#[clappen::clappen(export = endpoint)]
mod endpoint {
    pub struct Endpoint { pub url: String }
    #[clappen_template_impl]
    impl From<Prefixed> for Base {
        fn from(value: Prefixed) -> Self {
            Self { url: value.url }
        }
    }
    // the reverse: Endpoint is flattened in a parent with a template
    #[clappen_template_impl]
    impl From<Base> for Prefixed {
        fn from(value: Base) -> Self {
            Self { url: value.url }
        }
    }
}

#[clappen::clappen(export = server)]
mod server {
    pub struct Server {
        pub name: String,
        #[clappen_command(apply = endpoint, prefix = "api")]
        pub backend: Endpoint,
    }
    // a regular (non-template) impl
    impl Server {
        pub fn label(&self) -> &str { &self.name }
    }
    #[clappen_template_impl]
    impl From<Prefixed> for Base {
        fn from(value: Prefixed) -> Self {
            Self { name: value.name, backend: Endpoint::from(value.backend).into() }
        }
    }
}
```

The templates name the root `Endpoint`, so `Endpoint` must be in scope where `server!()` and
`server!("test1")` run. An `endpoint!()` call creates it.

**`#[clappen]` builds one macro per mod.** No code is emitted yet. The `clappen` proc-macro
turns each mod into a `macro_rules!`. The sketch for `server`:

```rust
macro_rules! server {
    () => { /* 1. struct, 2. regular impls, 4. child_apply */ };
    ($prefix: literal) => { /* 1. struct, 2. regular impls, 3. self_apply, 4. child_apply */ };
    (@__struct $($prefix: literal)?) => { /* 1. struct, 2. regular impls */ };
    (@__template $($prefix: literal,)? chain = [ .. ]) => { /* 3. self_apply, 4. child_apply */ };
}
```

`server!()` and `server!("test1")` are two independent invocations, each its own path through this
macro. The two sections below are alternatives, not a sequence.

### `server!()` (base)

Matches `()`, emitting items **1.**, **2.** and **4.**: the struct (tagged for `__clappen_struct`), the
regular impl (tagged for `__clappen_impl`), and the call into the child. Item **3.** is absent:

```rust
// 1. the struct. __clappen_struct fills its nested module via endpoint!(@__struct "api")
#[clappen::__clappen_struct]
pub struct Server {
    pub name: String,
    #[clappen_command(apply = endpoint, prefix = "api")]
    pub backend: Endpoint,
}

// 2. the regular impl (no prefix and no default_prefix here, so its tag changes nothing)
#[clappen::__clappen_impl(...)]
impl Server {
    pub fn label(&self) -> &str { &self.name }
}

// 3. self_apply: none (Prefixed and Base are the same type at the base)
// 4. child_apply: endpoint's templates, for the copy of Endpoint that Server holds
endpoint!(@__template chain = [("api", backend)]);
```

After the proc-macros run (no prefix and no `default_prefix`, so no renaming), the result is:

```rust
// 1. the struct, with its nested module
pub(crate) mod __inner_backend {
    pub struct ApiEndpoint { pub api_url: String }
}
pub struct Server {
    pub name: String,
    pub backend: __inner_backend::ApiEndpoint,
}

// 2. the regular impl
impl Server {
    pub fn label(&self) -> &str { &self.name }
}

// from 4. (child_apply): endpoint's two templates, Base = the root Endpoint
impl From<__inner_backend::ApiEndpoint> for Endpoint {
    fn from(value: __inner_backend::ApiEndpoint) -> Self {
        Self { url: value.api_url }
    }
}
impl From<Endpoint> for __inner_backend::ApiEndpoint {
    fn from(value: Endpoint) -> Self {
        Self { api_url: value.url }
    }
}
```

### `server!("test1")` (prefixed)

Matches `($prefix)` with `$prefix = "test1"`, emitting all four items:

```rust
// 1. the struct. An endpoint!(@__struct ...) call fills its nested module
#[clappen::__clappen_struct(prefix = "test1")]
pub struct Server {
    pub name: String,
    #[clappen_command(apply = endpoint, prefix = "api")]
    pub backend: Endpoint,
}

// 2. the regular impl
#[clappen::__clappen_impl(prefix = "test1", ...)]
impl Server {
    pub fn label(&self) -> &str { &self.name }
}

// 3. self_apply: this struct's own template impl
#[clappen::__clappen_template_impl(prefix = "test1", chain = [], ...)]
impl From<Prefixed> for Base {
    fn from(value: Prefixed) -> Self {
        Self { name: value.name, backend: Endpoint::from(value.backend).into() }
    }
}

// 4. child_apply: recurse into the flattened backend field
endpoint!(@__template "test1", chain = [("api", backend)]);
```

Item **4.** recurses into `endpoint`: that call runs `endpoint`'s `@__template` arm, which emits
`endpoint`'s own items **3.** and **4.** one level down. Item **3.** (`self_apply`) uses the inherited chain:

```rust
// 3. self_apply: endpoint's own two template impls, at the inherited chain
#[clappen::__clappen_template_impl(prefix = "test1", chain = [("api", backend)], ...)]
impl From<Prefixed> for Base { /* ... */ }
#[clappen::__clappen_template_impl(prefix = "test1", chain = [("api", backend)], ...)]
impl From<Base> for Prefixed { /* ... */ }
// 4. child_apply: none (endpoint has no flattened fields)
```

The three proc-macros then rewrite the tagged items **1.** to **3.** (item **4.** was the macro call above,
which expanded into `endpoint`'s items):

- `__clappen_struct` (**1.**) renames the struct and prefixes its fields (`Server` -> `Test1Server`,
  `name` -> `test1_name`), and for each `clappen_command` field emits the `__inner_*` module that
  calls the child's `@__struct`.
- `__clappen_impl` (**2.**) rewrites the regular impl: it renames the `Self` type and prefixes field accesses
  (`impl Server` -> `impl Test1Server`, `self.name` -> `self.test1_name`).
- `__clappen_template_impl` (**3.**) replaces the `Prefixed`/`Base` tags with concrete types that
  `ResolvedTag::new` computes from the chain. `server`'s impl has an empty chain, so `Prefixed = Test1Server`,
  `Base = Server`. `endpoint`'s impls have chain `[("api", backend)]`, the path to the nested
  copy, so `Prefixed = __inner_test1_backend::Test1ApiEndpoint`. `Base` is the root at every depth,
  so `Base = Endpoint`.

**Result.** `server!("test1")` has produced the prefixed struct, its regular impl, and three
conversions:

```rust
// 1. the struct, with its nested module
pub(crate) mod __inner_test1_backend {
    pub struct Test1ApiEndpoint { pub test1_api_url: String }
}
pub struct Test1Server {
    pub test1_name: String,
    pub test1_backend: __inner_test1_backend::Test1ApiEndpoint,
}

// 2. the regular impl
impl Test1Server {
    pub fn label(&self) -> &str { &self.test1_name }
}

// 3. self_apply
impl From<Test1Server> for Server {
    fn from(value: Test1Server) -> Self {
        Self { name: value.test1_name, backend: Endpoint::from(value.test1_backend).into() }
    }
}

// from 4. (child_apply): endpoint's own self_apply, Base = the root Endpoint
impl From<__inner_test1_backend::Test1ApiEndpoint> for Endpoint {
    fn from(value: __inner_test1_backend::Test1ApiEndpoint) -> Self {
        Self { url: value.test1_api_url }
    }
}
impl From<Endpoint> for __inner_test1_backend::Test1ApiEndpoint {
    fn from(value: Endpoint) -> Self {
        Self { test1_api_url: value.url }
    }
}
```

The regular impl's `self.name` now reads `self.test1_name`. The parent's
`Endpoint::from(value.test1_backend).into()` converts in two steps, through the root. The first step
uses the forward impl from `server!("test1")`. The second step uses the reverse impl from `server!()`,
which gives the `__inner_backend::ApiEndpoint` that `Server` holds.

## 4. Two kinds of generated code: `self_apply` and `child_apply`

Everything the template feature emits is one of two kinds:

- **`self_apply`** (**3.**): this struct's own template impl, emitted as a tagged
  `#[clappen::__clappen_template_impl(...)]` and rewritten in place, the same way `#[__clappen_impl]`
  handles a regular impl. No recursion.
- **`child_apply`** (**4.**): a call `CHILD!(@__template prefix, chain + step)` into a flattened
  child's macro, which makes the child emit its own `self_apply`.

Only `child_apply` needs the `@__template` arm, the recursion, and the chain. A regular impl only
touches `self.field`, which `__clappen_impl` rewrites in place. But a template conversion can do
`Endpoint::from(value.backend).into()`, and that needs `From<Test1ApiEndpoint> for Endpoint` and
`From<Endpoint> for ApiEndpoint` to exist first. Only `endpoint`'s own macro can generate them (the
parent's proc-macro has neither `endpoint`'s struct nor its templates), so the parent calls the
child's macro and passes the chain the child uses to place its nested types.

A template converts between a copy and its root, never between two copies. A direct impl between two
copies must name both of them at one place. But each copy sits in the `__inner_*` module of its own
call site, and another call site does not know the path of that module. So a nested field converts
in two steps, through the root.

Which arm emits which:

| arm | `self_apply` | `child_apply` |
|---|---|---|
| `()` base | no | yes |
| `($prefix)` prefixed | yes | yes |
| `(@__struct ...)` (a nested field's type) | no | no |
| `(@__template ...)` (this struct flattened in a parent) | yes | yes |

`base` has no `self_apply`: at the base arm, `Prefixed` and `Base` name the same type, so the impl
would be `From<T> for T`, which core already provides.

## 5. The helper proc-macros

- `__clappen_struct`: rewrites the struct, applies the field/struct prefix, and for each
  `clappen_command` field emits the `__inner_*` module that calls the child macro.
- `__clappen_impl`: renames the `Self` type and applies the prefix to an impl block's field accesses.
- `__clappen_template_impl`: rewrites one `#[clappen_template_impl]` block. It replaces the
  `Prefixed` tag with the copy at the path in the chain, and the `Base` tag with the root type.
