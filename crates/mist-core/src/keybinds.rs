use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize, Debug)]
/// The raw representation of mist keybinds as [`String`]s.
pub struct KeybindsRaw {
    pause: String,
    reset: String,
    start_split: String,
    prev_comp: String,
    next_comp: String,
    load_splits: String,
}

impl Default for KeybindsRaw {
    fn default() -> Self {
        KeybindsRaw {
            pause: "Return".to_owned(),
            reset: "R".to_owned(),
            start_split: "Space".to_owned(),
            prev_comp: "Left".to_owned(),
            next_comp: "Right".to_owned(),
            load_splits: "F1".to_owned(),
        }
    }
}
