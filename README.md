# <img src="assets/MIST.png" width="50" height="50"/> mist
![Lines of code](https://img.shields.io/tokei/lines/github/ltperiwinkle/mist)

a minimal, improved speedrun timer

[Changelog](CHANGELOG.md) \
[Latest Release](https://github.com/LtPeriwinkle/mist/releases/latest)

## Planned features
Once all planned features are complete, I will likely stop developing this and only squash bugs etc. More *might* still be added to this list.
- [X] Cross platform
	* [X] Linux
	* [X] Windows
	* [X] MacOS (compiled & tested in macOS catalina VM)
- [X] Human-readable split file (using [ron](https://github.com/ron-rs/ron))
- [X] LiveSplit split file transposing (split tool)
- [ ] (limited) customizability 
	* [X] custom fonts/font sizes
	* [X] custom colors
	* [X] keybinds
	* [X] timer backgrounds
	* [ ] panels (sum of best etc)
	* [ ] time rounding (30/60/off)
	* [ ] *very limited* timer layout (i.e. use two rows for splits like option available in LiveSplit)
- [ ] split file creation tool
	* [X] edit existing msf
	* [X] convert lss to msf
	* [X] create new splits
	* [ ] actually good and usable (hardest part)
- [X] fps-based time conversion (so that the timer always ends on a time that corresponds to a possible frame time) (30fps done)
- [X] dynamic colors
- [X] different run comparisons
	* [X] sum of best
	* [X] pb
	* [X] none
	* [X] average
- [X] hot reloading
	* [X] split file reloading
	* [X] config reloading
- [ ] plugins
	* [ ] autoloading from plugins directory (probably run as some kind of child process thing?)
	* [ ] communicate with plugins through ipc (i.e. unix socket, windows named pipe)
	* [ ] plugins that are shipped with this repo (a discord presence, some kind of notes plugin, maybe more)
- [ ] search for config/assets in standard os-specific dirs rather than hard-coded one (allows for packaging, installation, etc)
- [ ] better way to find fonts than paths in config file
- [X] skip splits (because somehow i missed this all along)

## Unplanned features
These features will not be implemented, in the spirit of minimalism.
* Autosplitters
* Horizontal timer layout
* Ingame time
* Internet time sync
* SRC/SRL/splits.io/racetime.gg integration
* GIFs
* Split icons

# Installation

## Compiling from source
Probably the best way to try this out is to compile it from source. To do this you need rust installed, and an installation guide
for that can be found [here](https://www.rust-lang.org/tools/install).

## Features
This package provides two features, `bg` and `icon`. `icon` sets the icon of the application when it is running, and requires sdl2_image.
`bg` allows for configuration of a background image, and requires both sdl2\_image and sdl2\_gfx. To use only `icon` (removing gfx requirement),
append
```
--no-default-features --features=icon
```

to the cargo commands below. For only `bg`, do the same except replace `icon` with `bg`. Finally, to remove both, remove the `--features` altogether.


When you run mist, make sure it is in the same directory as the `assets` directory or else it won't work.
### Linux
Requirements are SDL2, SDL2\_Image and SDL2\_TTF shared libraries, as well as development libraries. On ubuntu:
```
sudo apt-get install libsdl2-2.0.0 libsdl2-ttf-2.0.0 libsdl2-image-2.0.0 libsdl2-gfx-2.0.0 libsdl2-dev libsdl2-ttf-dev libsdl2-image-dev libsdl2-gfx-dev
```

On arch:
```
sudo pacman -S sdl2 sdl2_ttf sdl2_image sdl2_gfx
```

Clone this repo (`git clone https://github.com/LtPeriwinkle/mist`), enter the directory, and run `cargo build --release`. Move the
resulting binary from `./target/release/` into the repository root (or just the same folder as `assets/`) to run.

### Windows
Follow [this guide](https://github.com/Rust-SDL2/rust-sdl2#windows-msvc) to set up your SDL dependencies. You will have to follow this process for SDL2, SDL\_Image and SDL2\_TTF,
whose development stuff is available [here](http://libsdl.org/projects/SDL_ttf/) and [here](http://libsdl.org/projects/SDL_image). I had to use vcpkg to get sdl_gfx and then copy the .lib file to the
folder specified by in the guide.

Compile with `cargo build --release` then move the exe as well as the sdl related dlls into the same folder as the assets folder to run it.

### MacOS
Install sdl2, sdl image and sdl ttf. Using homebrew:
```
brew install sdl2 sdl2_image sdl2_ttf sdl2_gfx
```

Then you should be able to run `cargo build --release`.

# Usage
The default keybinds are: \
<kbd>F1</kbd>: Open new split file \
<kbd>Space</kbd>: Start/split/stop \
<kbd>Enter</kbd>: Pause \
<kbd>R</kbd>: Reset \
<kbd>&leftarrow;</kbd>: Previous comparison \
<kbd>&rightarrow;</kbd>: Next comparison \
Mousewheel: Scroll splits up/down (if there are more than fit in the window)

Mist reads configuration info from assets/mist.cfg in the directory where its executable is located.

## Credits
Thanks to [Xeryph](https://twitch.tv/xeryph1) and [Komali](https://youtube.com/c/KomaliPrinceOfRito) for testing, bug reports,
and help on things.

## Licensing
Like Rust itself, mist is licensed under MIT or Apache 2.0, at your option.
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.
