Development
===========

Dependencies
------------

Add dependencies with `cargo add` instead of manually editing `Cargo.toml`.

Imports
-------

Don't rename imports in `use` statements.

Imports which are unambiguously named and common should go in the top-level
module and be inherited by child modules with `use super::*`.

Style
-----

Don't create mutable variables that are initialized in a conditional:

```rust
let mut foo = None;

if bar {
  foo = Some("hello");
}
```

Instead, create an immutable variable initialized with an if/else:

```rust
let foo = if bar {
  Some("hello")
} else {
  None
};
```

Passing primitives into functions creates opportunities for confusion:

```rust
struct Config {
  a: bool,
  b: bool,
}

fn foo(a: bool) {
}

let config = Config {
  a: true,
  b: false,
};

foo(config.b);
```

Where possible, pass the object where the primitive originates:

```rust
struct Config {
  a: bool,
  b: bool,
}

fn foo(config: &Config) {
  // use config.a
}

let config = Config {
  a: true,
  b: false,
};

foo(&config);
```
