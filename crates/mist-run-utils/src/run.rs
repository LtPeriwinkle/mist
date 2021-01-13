#[cfg(feature = "msf")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "msf", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct Run {
    game_title: String,
    category: String,
    offset: Option<u128>,
    pb: u128,
    splits: Vec<String>,
    pb_times: Vec<u128>,
    gold_times: Vec<u128>,
}

impl Run {
    pub fn new(game_title: String, category: String, offset: Option<u128>, pb: u128, splits: Vec<String>, pb_times: Vec<u128>, gold_times: Vec<u128>) -> Self {
	Run {
		game_title,
		category,
		offset,
		pb,
		splits,
		pb_times,
		gold_times
    	}
    }
    pub fn set_times(&mut self, splits: &Vec<u128>) {
        self.pb_times = splits.to_vec();
    }
    pub fn get_times(&self) -> &Vec<u128> {
        &self.pb_times
    }
    pub fn pb(&self) -> u128 {
        self.pb
    }
    pub fn set_pb(&mut self, pb: u128) {
        self.pb = pb;
    }
    pub fn get_golds(&self) -> &Vec<u128> {
	&self.gold_times
    }
    pub fn gold_time(&self, index: usize) -> u128 {
        self.gold_times[index]
    }
    pub fn set_gold_time(&mut self, index: usize, time: u128) {
        self.gold_times[index] = time;
    }
    pub fn offset(&self) -> Option<u128> {
        self.offset
    }
    pub fn split_names(&self) -> &Vec<String> {
        &self.splits
    }
}

/// create an empty run
impl Default for Run {
    fn default() -> Self {
        Run {
            game_title: "".to_string(),
            category: "".to_string(),
            offset: None,
            pb: 0,
            splits: vec![],
            pb_times: vec![],
            gold_times: vec![],
        }
    }
}
