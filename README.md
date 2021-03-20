# <img src="assets/MIST.png" width="50" height="50"/> mist
![Lines of code](https://img.shields.io/tokei/lines/github/ltperiwinkle/mist)
a minimal, improved speedrun timer

[Changelog](CHANGELOG.md) \
[Latest Release](https://github.com/LtPeriwinkle/mist/releases/latest)

## Planned features
More will probably be added to this list in the future
- [X] Cross platform
	* [X] Linux
	* [X] Windows
	* [X] MacOS (compiled & tested in macOS catalina VM)
- [X] Human-readable split file (using [ron](https://github.com/ron-rs/ron))
- [X] LiveSplit split file transposing
- [ ] (limited) customizability 
	* [X] custom fonts/font sizes
	* [X] custom colors
	* [ ] keybinds
	* [ ] timer backgrounds (maybe)
- [ ] split file creation tool
	* [X] edit existing msf
	* [X] convert lss to msf
	* [ ] create new splits
- [X] fps-based time conversion (so that the timer always ends on a time that corresponds to a possible frame time) (30fps done)
- [X] dynamic colors
- [ ] different run comparisons
	* [X] sum of best
	* [X] pb
	* [X] none
	* [ ] average overall
	* [ ] last x runs average
- [ ] integrated notes (like SpeedGuidesLive)
- [ ] hot reloading
	* [X] split file reloading
	* [ ] config reloading

## Requested features
Features that people have asked for but i'm not sure if i can implement go here
* Gif split icons
* Gif timer background
* Subsplits

## Unplanned features
These features are *highly unlikely* be implemented, in the spirit of minimalism. This does not always mean they will *never* happen.
* Autosplitters
* horizontal timer layout
* Ingame time
* Internet time sync
* SRC/SRL/splits.io/racetime.gg integration

# Installation

## Compiling from source
Probably the best way to try this out is to compile it from source. To do this you need rust installed, and an installation guide
for that can be found [here](https://www.rust-lang.org/tools/install).

When you run mist, make sure it is in the same directory as the `assets` directory or else it won't work.
### Linux
Requirements are SDL2, SDL2\_Image and SDL2\_TTF shared libraries, as well as development libraries. On ubuntu:
```
sudo apt-get install libsdl2-2.0.0 libsdl2-ttf-2.0.0 libsdl2-image-2.0.0 libsdl2-dev libsdl2-ttf-dev libsdl2-image-dev
```

On arch:
```
sudo pacman -S sdl2 sdl2_ttf sdl2_image
```

Clone this repo (`git clone https://github.com/LtPeriwinkle/mist`), enter the directory, and run `cargo build --release`. Move the
resulting binary from `./target/release/` into the repository root (or just the same folder as `assets/`) to run.

### Windows
Follow [this guide](https://github.com/Rust-SDL2/rust-sdl2#windows-msvc) to set up your SDL dependencies. You will have to follow this process for SDL2, SDL\_Image and SDL2\_TTF,
whose development stuff is available [here](http://libsdl.org/projects/SDL_ttf/) and [here](http://libsdl.org/projects/SDL_image).

Compile with `cargo build --release` then move the exe as well as the sdl related dlls into the same folder as the assets folder to run it.

### MacOS
Install sdl2, sdl image and sdl ttf. Using homebrew:
```
brew install sdl2 sdl2_image sdl2_ttf
```

Then you should be able to run `cargo build --release`.

# Usage
The current default keybinds are:
F1: Open new split file
Space: Start/split/stop
Enter: Pause
R: Reset
Left Arrow: Previous comparison
Right Arrow: Next comparison
Mousewheel: Scroll splits up/down (if there are more than fit in the window)

Mist reads configuration info from assets/mist.cfg in the directory where its executable is located.

## Licensing
Like Rust itself, mist is licensed under MIT or Apache 2.0, at your option.
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.
