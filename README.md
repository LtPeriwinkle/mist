# <img src="assets/MIST.png" width="50" height="50"/> mist
a minimal, improved speedrun timer

[Changelog](CHANGELOG.md) \
[Latest Release](https://github.com/LtPeriwinkle/mist/releases/latest)

---

The goal of this project is not to have more features than LiveSplit or to look better than LiveSplit.
Rather, the goal is to be lighter and more efficient than LiveSplit and to maintain compatibility with 
many platforms.
## Planned features
More will probably be added to this list in the future
- [X] Cross platform
	* [X] Linux
	* [X] Windows
	* [X] MacOS (compiled & tested in macOS catalina VM)
- [X] Human-readable split file (using [ron](https://github.com/ron-rs/ron))
- [ ] LiveSplit split file transposing (for now, check out [this site](https://lsstomist.komali09.repl.co))
- [ ] (limited) customizability 
	* [X] custom fonts
	* [ ] custom colors
	* [ ] timer backgrounds (maybe)
- [ ] split file creation tool
- [X] fps-based time conversion (so that the timer always ends on a time that corresponds to a possible frame time) (30fps done)
- [X] dynamic colors
- [ ] different run comparisons (i.e. SOB, average, last 5 runs avg, pb)
- [ ] integrated notes (like SpeedGuidesLive)

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
* SRC/SRL/splits.io integration

# Installation

## Compiling from source
Probably the best way to try this out is to compile it from source. To do this you need rust installed, and an installation guide
for that can be found [here](https://www.rust-lang.org/tools/install).

When you run mist, make sure it is in the same directory as the `assets` directory or else it won't work.
### Linux
Requirements are SDL2, SDL2\_Image and SDL2\_TTF shared libraries, as well as development libraries. On ubuntu:
```
sudo apt-get install libsdl2 libsdl2-ttf libsdl2-image libsdl2-dev libsdl2-ttf-dev libsdl2-image-dev
```

(i think)

On arch:
```
sudo pacman -S sdl2 sdl2_ttf sdl2_image
```

Clone this repo (`git clone https://github.com/LtPeriwinkle/mist`), enter the directory, and run `cargo build --release`. The resulting binary will be in
`./target/release/`. 

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

## Credits

## Licensing
Like Rust itself, mist is licensed under MIT or Apache 2.0, at your option.
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.
