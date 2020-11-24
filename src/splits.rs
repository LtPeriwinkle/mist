use ron::de::from_reader;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;

// The struct that contains data about a speedrun.
// More fields will be added for other split time comparisons, like average and worst times.
#[derive(Debug, Deserialize, Serialize)]
pub struct Run {
    pub game_title: String,
    pub category: String,
    pub pb: u128,
    pub splits: Vec<String>,
    pub best_times: Vec<u128>,
}

impl Run {
    // parse a RON file into a run. Real error handling will come... eventually
    pub fn from_file(filename: &str) -> Self {
        let file = OpenOptions::new().read(true).open(filename).unwrap();
        let run: Self = from_reader(&file).unwrap();
        return run;
    }

    pub fn new() -> Self {
        Self {
            game_title: "".to_string(),
            category: "".to_string(),
            pb: 0,
            splits: Vec::new(),
            best_times: Vec::new(),
        }
    }
}
