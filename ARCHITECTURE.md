# How clappen works

`#[clappen]` does not expand to code directly. Applied to a `mod`, it generates a
`macro_rules! NAME`, and you invoke that macro (`NAME!()`, `NAME!("primary")`, ...) to generate a
prefixed copy of the struct and its impls. Three hidden proc-macros do the per-item work:
`__clappen_struct`, `__clappen_impl`, `__clappen_template_impl`.

## 1. The macro arms

Each `NAME!` invocation matches one arm:

- `NAME!()` (base): the base struct and its regular impls.
- `NAME!("p")` (prefixed): a prefixed copy of the struct, its regular impls, and its template impls.
- `NAME!(@__struct "p")`: the same as `("p")` but without the template impls (used to build a nested field's type).
- `NAME!(@__template "p", chain = [ .. ])`: this struct's template impls at a given chain (reached when a parent recurses into it).

`("p")` and `@__template` build the same thing (this struct's template impls, then recursion into
its flattened fields), differing only in the chain: `("p")` starts it empty, `@__template` continues
a parent's. So the natural way to write `("p")` would be to call `@__template` on itself:

```rust
($prefix: literal) => {
    // ... prefixed struct ...
    NAME!(@__template $prefix, chain = []);   // does NOT work cross-crate
};
```

That self-call fails because clappen has no portable way to name its own macro. Neither form works
in both same-crate and cross-crate use:

- **bare `NAME!`** resolves at the call site: fine when the macro is used in its own crate, but from
  crate B, `crate_a::NAME!("p")` expands a bare `NAME!` that isn't in scope there, so it errors with
  `cannot find macro NAME`.
- **`$crate::NAME!`** would resolve cross-crate, but Rust rejects it in the macro's own crate: `NAME`
  is itself produced by a macro expansion (the `#[clappen]` proc-macro), and a macro-expanded
  `#[macro_export]` macro of the current crate cannot be named by an absolute path. This is the
  `macro_expanded_macro_exports_accessed_by_absolute_paths` lint (`error: macro-expanded macro_export
  macros from the current crate cannot be referred to by absolute paths`), tracked at
  [rust-lang/rust#52234](https://github.com/rust-lang/rust/issues/52234).

So `("p")` can't call itself at all.

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

- the **first** is a proc-macro attribute, not a macro call, so there is nothing to resolve;
- the **second** does call another generated macro, but through the path the user wrote in
  `#[clappen_command(apply = ...)]` (`CHILD` above): a bare name for same-crate use, or `$crate::child`
  (or a full path) for cross-crate. The user picks a path that resolves in their crate, so clappen
  never names that child by a fixed path.

So clappen never names *itself*. `@__template` still exists, for a parent to call when it recurses into this struct.

## 2. Nesting and the chain

`#[clappen_command(apply = CHILD, prefix = p)]` on a field flattens `CHILD` under prefix `p`. The
same field makes two calls into `CHILD`'s macro:

- `CHILD!(@__struct ...)` builds the nested struct, in a `pub(crate) mod __inner_field`;
- `CHILD!(@__template prefix, chain + step)` generates the child's template impls.

The **chain** is the list of steps that records where a nested struct sits: one `ChainStep`
`(command_prefix, field, parent_default)` per flatten level, collected from the top struct down to
the nested one. `ResolvedTag::new` combines those steps into the nested type's module path and field
prefix (turning the `Prefixed` and `Base` tags into concrete types). An empty chain is a top-level
instantiation; a non-empty chain is a nested one. The macros
are defined once per `mod`; nesting is those macros calling each other, and the chain tells each
call its position.

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

Calling `App!("x")` reaches `Pool` two levels down, so `Pool`'s `@__template` call gets a two-step
chain, one step per flatten level, the top struct's step first:

```rust
pool!(@__template "x", chain = [
    // step = (command_prefix, field, parent_default); parent_default is the parent's own
    // default_prefix, "" here since neither App nor Db sets one
    ("db",   database, ""),   // App flattens Db   as `database`, prefix "db"
    ("pool", conn,     ""),   // Db  flattens Pool as `conn`,     prefix "pool"
]);
```

`ResolvedTag::new` combines those two steps to place `Pool`'s prefixed type at its nested module path
(each step adds one `__inner_*` level and one prefix segment). A top-level `App!("x")` starts with an
empty chain; each flatten level appends one more step on the way down.

## 3. End-to-end walkthrough

A leaf struct with a template, and a parent that flattens it, has a regular impl, and also has a
template:

```rust
#[clappen::clappen(export = endpoint)]
mod endpoint {
    pub struct Endpoint { pub url: String }
    #[clappen_template_impl]
    impl From<Prefixed> for Base {
        fn from(v: Prefixed) -> Self {
            Self { url: v.url }
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
        fn from(v: Prefixed) -> Self {
            Self { name: v.name, backend: v.backend.into() }
        }
    }
}
```

**`#[clappen]` builds one macro per mod.** No code is emitted yet. The `clappen` proc-macro
turns each mod into a `macro_rules!`; for `server`, sketched:

```rust
macro_rules! server {
    () => { /* 1. struct, 2. regular impls */ };
    ($prefix: literal) => { /* 1. struct, 2. regular impls, 3. self_apply, 4. child_apply */ };
    (@__struct $prefix: literal) => { /* 1. struct only */ };
    (@__template $prefix: literal, chain = [ .. ]) => { /* 3. self_apply, 4. child_apply */ };
}
```

`server!()` and `server!("test1")` are two independent invocations, each its own path through this
macro. The two sections below are alternatives, not a sequence.

### `server!()` (base)

Matches `()`, emitting only items **1.** and **2.**: the struct (tagged for `__clappen_struct`) and the
regular impl (emitted as-is, nothing to prefix at the base). Items **3.** and **4.** are absent:

```rust
// 1. the struct; __clappen_struct fills its nested module via endpoint!(@__struct "api")
#[clappen::__clappen_struct]
pub struct Server {
    pub name: String,
    #[clappen_command(apply = endpoint, prefix = "api")]
    pub backend: Endpoint,
}

// 2. the regular impl, emitted as-is (no tag: nothing to prefix at the base)
impl Server {
    pub fn label(&self) -> &str { &self.name }
}

// 3. self_apply: none (base is unprefixed, so no From<Prefixed>)
// 4. child_apply: none (child conversions come from the prefixed path below)
```

After `__clappen_struct` runs (no prefix, so no renaming), the result is:

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
```

### `server!("test1")` (prefixed)

Matches `($prefix)` with `$prefix = "test1"`, emitting all four items:

```rust
// 1. the struct; its nested module is filled by an endpoint!(@__struct ...) call
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
    fn from(v: Prefixed) -> Self {
        Self { name: v.name, backend: v.backend.into() }
    }
}

// 4. child_apply: recurse into the flattened backend field
endpoint!(@__template "test1", chain = [("api", backend, "")]);
```

Item **4.** recurses into `endpoint`: that call runs `endpoint`'s `@__template` arm, which emits
`endpoint`'s own items **3.** and **4.** one level down. Item **3.** (`self_apply`) uses the inherited chain:

```rust
// 3. self_apply: endpoint's own template impl, at the inherited chain
#[clappen::__clappen_template_impl(prefix = "test1", chain = [("api", backend, "")], ...)]
impl From<Prefixed> for Base { /* ... */ }
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
  `Base = Server`. `endpoint`'s impl has chain `[("api", backend, "")]`, which locates the nested
  types, so `Prefixed = __inner_test1_backend::Test1ApiEndpoint`, `Base = __inner_backend::ApiEndpoint`.

**Result.** `server!("test1")` has produced the prefixed struct, its regular impl, and two
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
        Self { name: value.test1_name, backend: value.test1_backend.into() }
    }
}

// from 4. (child_apply): endpoint's own self_apply
impl From<__inner_test1_backend::Test1ApiEndpoint> for __inner_backend::ApiEndpoint {
    fn from(value: __inner_test1_backend::Test1ApiEndpoint) -> Self {
        Self { api_url: value.test1_api_url }
    }
}
```

The regular impl's `self.name` now reads `self.test1_name`, and the parent's `backend.into()`
compiles because the last impl is exactly the conversion it needs.

## 4. Two kinds of generated code: `self_apply` and `child_apply`

Everything the template feature emits is one of two kinds:

- **`self_apply`** (**3.**): this struct's own template impl, emitted as a tagged
  `#[clappen::__clappen_template_impl(...)]` and rewritten in place, the same way `#[__clappen_impl]`
  handles a regular impl. No recursion.
- **`child_apply`** (**4.**): a call `CHILD!(@__template prefix, chain + step)` into a flattened
  child's macro, which makes the child emit its own `self_apply`.

Only `child_apply` needs the `@__template` arm, the recursion, and the chain. A regular impl only
touches `self.field`, which `__clappen_impl` rewrites in place. But a template conversion can do
`value.backend.into()`, and that needs `From<Test1ApiEndpoint> for ApiEndpoint` to exist first. Only
`endpoint`'s own macro can generate it (the parent's proc-macro has neither `endpoint`'s struct nor
its template), so the parent calls the child's macro and passes the chain the child uses to place its
nested types.

Which arm emits which:

| arm | `self_apply` | `child_apply` |
|---|---|---|
| `()` base | no | yes, only when this struct has no template of its own |
| `($prefix)` prefixed | yes | yes |
| `(@__template ...)` (this struct flattened in a parent) | yes | yes |

`base` has no `self_apply`: the base is unprefixed, so there is no `From<Prefixed>` for it.

## 5. The helper proc-macros

- `__clappen_struct`: rewrites the struct, applies the field/struct prefix, and for each
  `clappen_command` field emits the `__inner_*` module that calls the child macro.
- `__clappen_impl`: applies the prefix to an impl block's field accesses.
- `__clappen_template_impl`: specializes one `#[clappen_template_impl]` block, replacing the
  `Base`/`Prefixed` tags with the concrete types resolved from the chain.
