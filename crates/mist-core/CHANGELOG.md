## mist-core changelog
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
