[![continuous-integration](https://github.com/killzoner/clappen/actions/workflows/continuous-integration.yml/badge.svg)](https://github.com/killzoner/clappen/actions/workflows/continuous-integration.yml)

# clappen

> **Flatten prefix for `clap`**

## About

Integrate flatten prefix in your `clap` parsers easily.

For more details, see:

- [examples](https://github.com/killzoner/clappen/tree/master/examples)

## Basic usage

```rust no_run
use clap::Parser;

// export is the name of the macro generated
#[clappen::clappen(export = nested)]
// this mod definition does not appear in the
// final generated code, it's just a convenient
// wrapper for item definitions
mod unused_mod {
    #[derive(clap::Args, Debug, Clone)]
    pub struct Remote {
        #[arg(env, long)]
        id: String,
    }

    impl Remote {
        fn a_function(&self) -> String {
            // this `self` reference will be rewritten
            // to field prefixed with `test`, ie `self.test_id`.
            format!("id: {:?}", self.id)
        }
    }
}

// export is the name of the macro generated
#[clappen::clappen(export = prefixed_struct_generator)]
// mod definition not used too
mod unused_mod {
    #[derive(clap::Parser, Debug, Clone)]
    #[command(version, about)]
    pub struct Options {
        #[arg(env, long)]
        url: String,

        #[command(flatten)]
        // `apply` is the macro export used
        // (defined with `Remote` struct earlier)
        // `prefix` is optional
        #[clappen_command(apply = nested, prefix = "test")]
        nested: Remote,
    }
}

// generate the default struct without prefix
prefixed_struct_generator!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = Options::parse();

    Ok(())
}
```

## Motivation

`clap` unfortunately doesn't preserve field structure and prefix when flattening `command`s.\
See [this issue](https://github.com/clap-rs/clap/issues/3513) for more in-depth explanation.

This crate allows fixing that while not requiring custom wrapper to the end `clap` parser used. It's just a `struct` macro generator that uses `clap` default behaviour around `arg` attribute.\
\
See [clap documentation for arg](https://docs.rs/clap/latest/clap/_derive/index.html#arg-attributes).

## Limitations

- Providing custom `long` or `env` to your fields is not supported. See [Roadmap](https://github.com/killzoner/clappen?tab=readme-ov-file#roadmap).

- References to fields with more than 1 nesting level won't work - like `self.my_field_level1.my_field_level2`.\
    \
  Generally this should not be needed because you want to get something *out* of your reusable struct parser (and Rust might get in your way for borrow-related things).\
  \
  If you *still* want to do that, you can write custom getter functions, and the renamed fields will be picked in the `impl` blocks.

- Probably some edge cases are not covered. For example, `clap` `subcommand`s should be working, but it was not tested extensively as my use case is mainly reusing `struct`s in a mono repository fashion.

## Roadmap

- Maybe add support for `long` / `env` prefix injection.\
  This would make the integration with `clap` tighter and the code more complicated though.

## FAQ

### Why is `clappen_command` required with `command(flatten)` even without prefix  ?

Because people work with copy/paste.

For example, if this was not required, you might write this

```rust
#[clappen::clappen(export = nested)]
mod m {
    #[derive(clap::Args)]
    pub struct Nested {}
}

#[clappen::clappen(export = copyable_opts)]
mod m {
    #[derive(clap::Parser)]
    #[command(version, about)]
    pub struct CopyableOptions {
        #[command(flatten)]
        nested: Nested,
    }
}
```

But then if you copy paste this and turn it to a reusable parser, you get this

```rust
#[clappen::clappen(export = nested)]
mod m {
    #[derive(clap::Args)]
    pub struct Nested {}
}

#[clappen::clappen(export = copyable_opts)]
mod m {
    #[derive(clap::Args)]
    pub struct CopyableOptions {
        #[command(flatten)]
        nested: Nested,
    }
}

#[clappen::clappen(export = parser)]
mod m {
    #[derive(clap::Parser)]
    #[command(version, about)]
    pub struct Options {
        #[command(flatten)]
        // You're back to original issue, prefix is 
        // not maintained because `clappen` was not provided
        opts: CopyableOptions,
    }
}
```

Making `clappen_command` required for all `flatten` items avoids having to think about that when refactoring, you know that your prefix will be maintained even when using a single struct without a prefix.

## Related stuff

- Project <https://github.com/wfraser/clap_wrapper> - implements prefix as well, but you can't reuse the struct with another prefix.
- Issue <https://github.com/clap-rs/clap/issues/3513#issuecomment-1344372578> for the original idea.

## Special thanks

- Big up to [@dtolnay](https://github.com/dtolnay) and its wonderful work in `syn`, `quote` and `paste`.\
  The initial implementation used `paste` but it is sadly now [unmaintained](https://github.com/dtolnay/paste/commit/6a302522990cbfd9de4e0c61d91854622f7b2999).
- Kudos to [@epage](https://github.com/epage) for tireless maintaining `clap` and all its great features.
