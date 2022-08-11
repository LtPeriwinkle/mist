# mist-core

![Crates.io](https://img.shields.io/crates/v/mist-core)

The heart of functionality of [mist](https://codeberg.org/LieutenantPeriwinkle/mist). Could also be used to create another speedrun
timer, although this is inadvisable.

## usage (why)
Add mist-core to your Cargo.toml.

```toml
[dependencies.mist-core]
version = "0.9"
```

`mist-core` provides several features: `dialogs`, `config`, `lss`, `instant`, and `bg`.
`bg` is used by mist to enable or enable background image support in configuration; `dialogs`, and `config` enable their respective
modules; `lss` adds the LssParser to module parse; `instant` enables an alternate `Instant` implementation on
platforms where the version from `std` does not already measure time how I want them to.
