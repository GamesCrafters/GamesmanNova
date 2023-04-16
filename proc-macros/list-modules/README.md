# List-Modules Procedural Macro

This macro creates a constant string slice list of all the module names
which are children of an indicated crate module folder. Paths are specified
relative to the cargo manifest directory.

For example, calling this macro from `mod.rs` in the following file tree
with `list_modules::here!("parent/");`...

```none
parent/
    mod.rs
    child_1.rs
    child_2/
        mod.rs
        internal.rs
        other_internal/
            ...
        ...
    child_3.rs
    child_4.rs
    ...
    child_N/
        mod.rs
```

...will result in the following list expansion:

```rust
pub const LIST: [&str; N] = [
    "child_1",
    "child_2",
    "child_3",
    ...
    "child_N",
];
```

Note that this is the only guaranteed behavior.
