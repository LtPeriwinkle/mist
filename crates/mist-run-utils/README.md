# mist-run-utils

library for parsing [mist](https://github.com/LtPeriwinkle/mist) split files and converting livesplit
files to mist runs. also defines interaction with the run struct, used in mist and mist-split-tool

to use, put in your Cargo.toml:
```toml
[dependencies.mist-run-utils]
version = "2.1"
features = ["msf", "lss"]
```
The default features are none, which only gets you the Run struct and associated methods.
msf and lss features include parsing for msf and lss split files respectively. you can use either or both.
