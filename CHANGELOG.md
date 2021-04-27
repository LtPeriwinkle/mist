# Changelog
sorry if the versioning doesnt really make sense, ive been trying to follow semver but i keep messing up
### 1.9.0
- fixed some warnings that have been around forever
- fixed an issue where diffs would never be set to gold
- added dialog boxes that pop up when unwrap is called on an err

### 1.8.0
- background image finally added
- added config field for whether to crop the center of the image or to scale the image to fit the window

### 1.7.0
- implement large timer font rendering from a cache texture of all the characters to avoid recreating a new texture each frame
- make millisecond display on large timer smaller so that longer times fit better on the screen and look nicer
- finally get rid of text jitter when updating large timer on some weird fonts

### 1.6.5
- fix incorrect color calculation after pausing and unpausing
- fix weird diff calculation after pausing

### 1.6.4
- prompt to save run on exit if a gold occurred

### 1.6.3
- fix crash when timer offset is None
- fix incorrect behavior when a new split file is loaded that has a different number of splits than the previous

### 1.6.2
- fix timer overlapping splits when window is shrunk vertically

### 1.6.1
- rearrange some code in App::init() and run()
- patch index-related issues when hot-reloading a split file with a different number of splits
- fix an issue where splits decreased when increasing the window size

### 1.6.0
- make config loading more logical
- only reads `assets/mist.cfg` now for consistency
- removed terrible config select dialog boxes on startup
- dynamically determine max splits on startup based on the window size and font size

### 1.5.4
- finally add customizable colors from config file

### 1.5.3
- turn off kerning in order to fix timer jitter with certain fonts

### 1.5.2
- increment index variables in loops when ending a run so it doesn't get stuck (i'm stupid)

### 1.5.1
- move check to save run to after user has closed program

### 1.5.0
- add comparisons to PB, golds, and empty
- calculate colors for timer based on comparison
- switch comparisons with arrow keys
- change displayed split times with comparison swap

### 1.4.0
- convert to using run from [mist-split-utils](https://github.com/LtPeriwinkle/mist-split-utils)
- clean up the code used to open and validate split files on startup

### 1.3.3
- patch issues with 0-split files

### 1.3.2
- add font size field in config
- allow user selected config, default to assets/default.mts if none
- create config file if missing

### 1.3.1
- add separate fields for timer and split font in config struct
- use font paths from config in app.rs

### 1.3.0
- add configuration file and cfg file parsing
	* config file holds last opened run, colors for timer, path to font
	* colors dont work yet but they will soon
	* custom config not yet selectable
	* will be selectable along with new run when context menu is implemented
- properly save golds on run end

### 1.2.8
- first crates.io published working version
- had to increment version cause i'm stupid

### 1.2.7
- hopefully patch windows file filtering
- add golds for real

### 1.2.6
- reset to top of splits on timer reset
- add preliminary golds suppord
- add proper error handling to msf file parsing

### 1.2.5
- ask to save after rendering last frame (looks much nicer this way)
- on pb, properly update current and pb times and textures of Splits in memory
- only actually save times to chosen file if user agrees to
- fix zero padding, remove extraneous decimals on split times

### 1.2.4
- require split file input path
- patch issue where all splits would happen instantly if you hold down split key

### 1.2.3
- add tinyfiledialog dependency
- add yes/no save splits dialog for writing to msf file
- save run on run end not on splits scroll like a *fool*

### 1.2.2
- fix highlighting the current split when scrolling
- display the proper time when the run ends
- condense some match patterns

### 1.2.1
- properly calculate diffs
- tweak color values

### 1.2.0
- patch color calculation hopefully for the last time
- render diff textures with '+' when behind
- account for pausing in color calculation
- properly clear old textures on timer reset

### 1.1.3
- add split time diff rendering
	* currently no way to handle horizontal resize
	* dynamic color might still be wrong unfortunately

### 1.1.2
- fix dynamic timer color calculation
	* now properly uses making up time color and losing time color
	* still breaks after a pause, will be fixed in a later patch as pausing isnt horribly common

### 1.1.1
- use instant everywhere instead of SDL timer
	* this reduces the number of u32 -> u128 casts
	* also just feels nicer

### 1.1.0
- massive internal changes to split system
	* now uses a wrapper struct for splits to reduce clutter
	* no longer requires large numbers of lifetime-dodging kludges
	* properly implemented `Split` struct field accessing

### 1.0.0
- Basic speedrun timing functionality
- Start offset support
- Read run from split file (file currently locked to "run.msf" in directory where executable is stored)
- If completed run is a PB, save run data to split file
- Change timer color according to run status (not sure if this all works properly)
- Spacebar to start, split, stop; Enter to pause/unpause; R key to reset timer
- Convert time to 30fps values on stop (non-configurable)
- Doesnt crash when you resize the window vertically (yay!) (horizontal resizes probably still bad)
