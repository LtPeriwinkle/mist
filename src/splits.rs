use std::fs::OpenOptions;
use ron::de::from_reader;
use serde::{Deserialize, Serialize};

// The struct that contains data about a speedrun.
// More fields will be added for other split time comparisons, like average and worst times.
#[derive(Debug, Deserialize, Serialize)]
pub struct Run {
	game_title: String,
	category: String,
	pb: u128,
	splits: Vec<String>,
	best_times: Vec<u128>
}

impl Run {
    	// parse a RON file into a run. Real error handling will come... eventually
	pub fn from_file(filename: &str) -> Self {
		let file = OpenOptions::new().read(true).open(filename).unwrap();
		let mut run: Self = from_reader(&file).unwrap();
		return run;
	}
}

// will do something eventually
pub fn get_splits() -> Vec<&'static str> {
    vec![
        "Something",
        "else",
        "words",
        "text",
        "split 5 idk",
        "q",
        "asdf",
        "words 2",
        "no",
        "yes",
        "another one",
    ]
}

pub fn get_split_times() -> Vec<u128> {
    vec![
        1000, 2000, 3000, 4000, 5000, 6000, 7000, 8000, 9000, 10000, 11000,
    ]
}
