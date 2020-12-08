# Changelog
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
