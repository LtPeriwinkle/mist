use super::TimeType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
/// Holds information about a speedrun and a user's times.
pub struct Run {
    game_title: String,
    category: String,
    offset: TimeType,
    pb: TimeType,
    splits: Vec<String>,
    pb_times: Vec<TimeType>,
    gold_times: Vec<TimeType>,
    sum_times: Vec<(u128, TimeType)>,
}

impl Run {
    /// Create a [`Run`] with all empty fields.
    pub fn empty() -> Self {
        Run {
            game_title: String::new(),
            category: String::new(),
            offset: TimeType::None,
            pb: TimeType::None,
            splits: vec![],
            pb_times: vec![],
            gold_times: vec![],
            sum_times: vec![],
        }
    }
    /// Create a new [`Run`].
    #[allow(clippy::too_many_arguments)]
    pub fn new<S>(
        game_title: S,
        category: S,
        offset: TimeType,
        pb: TimeType,
        splits: &[String],
        pb_times: &[TimeType],
        gold_times: &[TimeType],
        sum_times: &[(u128, TimeType)],
    ) -> Self
    where
        S: ToString,
    {
        Run {
            game_title: game_title.to_string(),
            category: category.to_string(),
            offset,
            pb,
            splits: splits.to_owned(),
            pb_times: pb_times.to_owned(),
            gold_times: gold_times.to_owned(),
            sum_times: sum_times.to_owned(),
        }
    }
    /// Get the game title.
    pub fn game_title(&self) -> &str {
        &self.game_title
    }
    /// Get the speedrun category.
    pub fn category(&self) -> &str {
        &self.category
    }
    /// Get start offset of run in milliseconds. None means no offset.
    pub fn offset(&self) -> TimeType {
        self.offset
    }
    /// Get the pb of the run in ms.
    pub fn pb(&self) -> TimeType {
        self.pb
    }
    /// Returns the split names in the run.
    pub fn splits(&self) -> &Vec<String> {
        &self.splits
    }
    /// Returns the times that were set on each split on the last personal best.
    pub fn pb_times(&self) -> &Vec<TimeType> {
        &self.pb_times
    }
    /// Returns the times that were set on each split on the last personal best, as their u128 values.
    pub fn pb_times_u128(&self) -> Vec<u128> {
        self.pb_times.iter().map(|t| t.raw()).collect()
    }
    /// Returns the best time that the runner has achieved on each split.
    pub fn gold_times(&self) -> &Vec<TimeType> {
        &self.gold_times
    }
    /// Returns the best time that the runner has achieved on each split, as their u128 values.
    pub fn gold_times_u128(&self) -> Vec<u128> {
        self.gold_times.iter().map(|t| t.val()).collect()
    }
    /// Returns tuples of attempt count and total number of milliseconds spent for each split.
    ///
    /// First element is attempt count and second is the total time;
    /// useful for calculating averages.
    pub fn sum_times(&self) -> &Vec<(u128, TimeType)> {
        &self.sum_times
    }
    /// Sets the game title.
    pub fn set_game_title<S>(&mut self, new: S)
    where
        S: ToString,
    {
        self.game_title = new.to_string();
    }
    /// Sets the name of the category.
    pub fn set_category<S>(&mut self, new: S)
    where
        S: ToString,
    {
        self.category = new.to_string();
    }
    /// Sets the start offset of the run.
    pub fn set_offset(&mut self, new: TimeType) {
        self.offset = new;
    }
    /// Set the pb of the run.
    pub fn set_pb(&mut self, new: TimeType) {
        self.pb = new;
    }
    /// Set the names of all splits.
    pub fn set_splits(&mut self, new: &[String]) {
        self.splits = new.to_owned();
    }
    /// Set the times for each split that were achieved on the current pb.
    pub fn set_pb_times(&mut self, new: &[TimeType]) {
        self.pb_times = new.to_owned();
    }
    /// Set the best time for each split.
    pub fn set_gold_times(&mut self, new: &[TimeType]) {
        self.gold_times = new.to_owned();
    }
    /// Set a single gold time, specified by `idx`.
    pub fn set_gold_time(&mut self, idx: usize, new: TimeType) {
        self.gold_times[idx] = new;
    }
    /// Set the attempt count and total time for all splits.
    /// First element is number of attempts of that split and second is the total time.
    pub fn set_sum_times(&mut self, new: &[(u128, TimeType)]) {
        self.sum_times = new.to_owned();
    }
    /// Set the attempt count and total for one split, specified by `idx`.
    pub fn set_sum_time(&mut self, idx: usize, new: (u128, TimeType)) {
        self.sum_times[idx] = new
    }
}
