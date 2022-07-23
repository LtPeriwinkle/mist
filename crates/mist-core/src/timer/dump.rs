use super::{state::SplitStatus, Comparison, DiffType, Run, TimeType};
use ron::{
    de::from_reader,
    ser::{to_string_pretty, PrettyConfig},
};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct StateDump {
    pub run: Run,
    pub status: SplitStatus,
    pub comparison: Comparison,
    pub run_times: Vec<TimeType>,
    pub run_diffs: Vec<DiffType>,
    pub run_golds: Vec<bool>,
    pub sum_comp_times: Vec<u128>,
    pub before_pause: u128,
    pub before_pause_split: u128,
    pub time: u128,
    pub current_split: usize,
    pub needs_save: bool,
    pub top_index: usize,
    pub bottom_index: usize,
    pub time_str: String,
}

impl StateDump {
    pub fn open<P: AsRef<Path>>(filename: P) -> Result<Self, String> {
        from_reader(File::open(filename).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
    }
    pub fn set_render_info(&mut self, top_index: usize, bottom_index: usize, time_str: String) {
        self.top_index = top_index;
        self.bottom_index = bottom_index;
        self.time_str = time_str;
    }
    pub fn print(&self) {
        println!("{}", to_string_pretty(self, PrettyConfig::new()).unwrap());
        std::fs::write(
            "foo.ron",
            to_string_pretty(self, PrettyConfig::new()).unwrap(),
        )
        .unwrap();
    }
}
