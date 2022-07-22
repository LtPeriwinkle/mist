use super::{state::SplitStatus, Comparison, DiffType, TimeType};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct StateDump {
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
    pub fn set_render_info(&mut self, top_index: usize, bottom_index: usize, time_str: String) {
        self.top_index = top_index;
        self.bottom_index = bottom_index;
        self.time_str = time_str;
    }
    pub fn print(&self) {
        println!(
            "{}",
            ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::new()).unwrap()
        );
        std::fs::write(
            "foo.ron",
            ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::new()).unwrap(),
        )
        .unwrap();
    }
}
