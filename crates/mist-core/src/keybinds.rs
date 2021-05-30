use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize, Debug)]
/// The raw representation of mist keybinds as [`String`]s.
pub struct KeybindsRaw {
    pub pause: String,
    pub reset: String,
    pub start_split: String,
    pub prev_comp: String,
    pub next_comp: String,
    pub load_splits: String,
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
