use mist_core::config::KeybindsRaw;
use sdl2::keyboard::Keycode;

#[derive(Debug)]
pub struct Keybinds {
    pub pause: Keycode,
    pub reset: Keycode,
    pub start_split: Keycode,
    pub skip_split: Keycode,
    pub un_split: Keycode,
    pub prev_comp: Keycode,
    pub next_comp: Keycode,
    pub load_splits: Keycode,
    pub load_config: Keycode,
}

impl Keybinds {
    pub fn from_raw(raw: &KeybindsRaw) -> Result<Self, String> {
        Ok(Keybinds {
            pause: Keycode::from_name(&raw.pause).ok_or("Pause keybind could not be parsed.")?,
            reset: Keycode::from_name(&raw.reset).ok_or("Reset keybind could not be parsed.")?,
            start_split: Keycode::from_name(&raw.start_split)
                .ok_or("start/split keybind could not be parsed.")?,
            skip_split: Keycode::from_name(&raw.skip_split)
                .ok_or("Skip split keybind could not be parsed")?,
            un_split: Keycode::from_name(&raw.un_split)
                .ok_or("Unsplit keybind could not be parsed")?,
            prev_comp: Keycode::from_name(&raw.prev_comp)
                .ok_or("Prev comparison keybind could not be parsed")?,
            next_comp: Keycode::from_name(&raw.next_comp)
                .ok_or("Next comparison keybind could not be parsed")?,
            load_splits: Keycode::from_name(&raw.load_splits)
                .ok_or("Load splits keybind could not be parsed")?,
            load_config: Keycode::from_name(&raw.load_config)
                .ok_or("Load config keybind could not be parsed")?,
        })
    }
}
