## mist-core changelog
### 1.1.0
- add StateDump struct for, well, dumping (and restoring) state
- various patches to RunState to allow restoring state accurately
- more things serde(default) for config files
- add keybinds to config for dump and restore

### 1.0.1
- add get_bytes function for fonts to work on all systems
- sanify the run upon reading it, as somehow this was missing

### 1.0.0
- Add many new config fields
- Allow fields to be unspecified, filling in with default
- Use platform-specific config locations
- Add exit confirmation dialog
- New timetype enum for skipping splits etc
- New font locating system for config
- Fix features

### 0.10.0
- Brand new state system (wow)
- Reorganized modules: `Run`, `RunState`, `MistInstant` etc all live in `timer` module
- Fix single-line comments in split files

### 0.9.0
- implement a custom `Instant` to measure time across system suspends

### 0.8.2
- check for general run sanity before returning a parsed run or writing a run to msf

### 0.8.1
- get font path rather than bytes of file to deal with lifetimes for rwops better

### 0.8.0
- add utility to load system fonts
- add font struct in config, specified whether path to file or system font.

### 0.7.0
- time rounding to arbitrary fps values
- changed config frame_rounding to `Option<u128>`

### 0.6.0
- move all config related stuff to its own module
- add layout options, time rounding, panels to config

### 0.5.0
- add keybinds for un/skip split as well as reload config
- add a dialog box to open a config file

### 0.4.3
- add setter for individual sum times that i forgot before

### 0.4.2
- fix conversion 

### 0.4.1
- make fields of keybinds pubilc

### 0.4.0
- add keybinds to config struct

### 0.3.1
- add missing newline to version writer

### 0.3.0
- make dialogs::get_file public
- add dialogs::get\_save\_as

### 0.2.0
- add parsing for legacy runs
- add run constructor with fields

### 0.1.0
- initial release
