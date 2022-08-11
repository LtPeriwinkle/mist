//! Snapshots of mist's state, for restoration.
use super::{state::SplitStatus, Comparison, DiffType, Run, TimeType};
use ron::{de::from_reader, ser::to_string};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;

#[derive(Serialize, Deserialize)]
/// A snapshot of mist's state.
///
/// This can be used for serialization to disk so that the user can stop and pick
/// up a run in the middle of the run, for example to restart a computer during a
/// break in a longer speedrun or something.
///
/// All fields are meaningful either in [`RunState`](super::state::RunState) or in the renderer.
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
    /// Open a `StateDump` from a file.
    ///
    /// Reads the file specified by `filename` and attempts to parse a `StateDump`
    /// from the contents. Returns `Err` on an fs error or if the file is not
    /// parseable.
    pub fn open<P: AsRef<Path>>(filename: P) -> Result<Self, String> {
        from_reader(File::open(filename).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
    }
    /// Set the information required from the renderer.
    pub fn set_render_info(&mut self, top_index: usize, bottom_index: usize, time_str: String) {
        self.top_index = top_index;
        self.bottom_index = bottom_index;
        self.time_str = time_str;
    }
    /// Serialize the `StateDump` to a file.
    ///
    /// Returns `Err` on fs error or if the dump could not be serialized.
    pub fn write<P: AsRef<Path>>(&self, filename: P) -> Result<(), String> {
        std::fs::write(filename, to_string(self).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())
    }
}
