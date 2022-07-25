use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
/// The raw representation of mist keybinds as strings.
pub struct KeybindsRaw {
    pub pause: String,
    pub reset: String,
    pub start_split: String,
    pub skip_split: String,
    pub un_split: String,
    pub prev_comp: String,
    pub next_comp: String,
    pub load_splits: String,
    pub load_config: String,
    pub dump_state: String,
    pub load_state: String,
}

impl Default for KeybindsRaw {
    fn default() -> Self {
        KeybindsRaw {
            pause: "Return".to_owned(),
            reset: "R".to_owned(),
            start_split: "Space".to_owned(),
            skip_split: "Right Shift".to_owned(),
            un_split: "Backspace".to_owned(),
            prev_comp: "Left".to_owned(),
            next_comp: "Right".to_owned(),
            load_splits: "F1".to_owned(),
            load_config: "F2".to_owned(),
            dump_state: "F3".to_owned(),
            load_state: "F4".to_owned(),
        }
    }
}
