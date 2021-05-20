# mist-core

![Crates.io](https://img.shields.io/crates/v/mist-core)

The heart of functionality of [mist](https://github.com/LtPeriwinkle/mist). Could also be used to create another speedrun
timer, although this is inadvisable.

## usage (why)
Add mist-core to your Cargo.toml.

```toml
[dependencies.mist-core]
version = "0.1"
```

`mist-core` provides several features: `timing`, `dialogs`, `config`, `lss`, and `bg`. These disable functionality if you don't want it.
`bg` is used by mist to enable or disable background image support in configuration; `timing`, `dialogs`, and `config` disable their respective
modules; `lss` removes the LssParser from module parse.
