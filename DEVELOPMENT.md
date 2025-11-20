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
