# <img src="assets/MIST.png" width="30" height="30"/> mist
a minimal, improved speedrun timer

[Changelog](CHANGELOG.md) \
[Latest Release](https://github.com/LtPeriwinkle/mist/releases/tag/v1.0.0)

---

The goal of this project is not to have more features than LiveSplit or to look better than LiveSplit.
Rather, the goal is to be lighter and more efficient than LiveSplit and to maintain compatibility with 
many platforms.
## Planned features
More will probably be added to this list in the future
- [ ] Cross platform
	* [X] Linux
	* [X] Windows
	* [X] MacOS (compiled & tested in macOS catalina VM)
- [X] Human-readable split file (using [ron](https://github.com/ron-rs/ron))
- [ ] LiveSplit split file transposing (for now, check out [this site](https://lsstomist.komali09.repl.co))
- [ ] (limited) customizability (i.e custom colors, perhaps fonts and layouts)
- [ ] split file creation tool
- [X] fps-based time conversion (so that the timer always ends on a time that corresponds to a possible frame time) (30fps done)
- [X] dynamic run comparisons and colors
- [ ] last 5 run avg comparison
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

## Compiling from source
Currently the only way to try this out is to compile it from source. To do this you need rust installed, and an installation guide
for that can be found [here](https://www.rust-lang.org/tools/install).
### Linux
Requirements are SDL2 and SDL2_TTF shared libraries, as well as development libraries. On ubuntu:
```bash
sudo apt-get install libsdl2 libsdl2-ttf libsdl2-dev libsdl2-ttf-dev
```

(i think)

On arch:
```bash
sudo pacman -S sdl2 sdl2_ttf
```

Clone this repo (`git clone https://github.com/LtPeriwinkle/mist`), enter the directory, and run `cargo build --release --locked`. The resulting binary will be in
`./target/release/`. When you run it, make sure it is in the same directory as `assets/` or else it won't work.

### Windows
Follow [this guide](https://github.com/Rust-SDL2/rust-sdl2#windows-msvc) to set up your SDL dependencies. You will have to follow this process for SDL2 and SDL2\_TTF,
whose development stuff is available [here](http://libsdl.org/projects/SDL_ttf/).

Clone the repository as shown in the linux section, and enter the folder. Then run `cargo build --release --locked`. The .exe will be in `.\target\release\`. Move it into
the same folder as `assets\`, as well as all of the SDL2 and SDL2\_TTF related dll files in order to run it.

## Licensing
Like Rust itself, mist is licensed under MIT or Apache 2.0, at your option.
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.
