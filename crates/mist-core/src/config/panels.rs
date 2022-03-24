use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
/// Different types of information panels.
///
/// `golds` field represents whether to compare against gold times rather than pb times.
pub enum Panel {
    /// Comparison to sum of runner's golds
    SumOfBest,
    /// Comparison to the current split, either gold or pb
    CurrentSplitDiff { golds: bool },
    /// Prediction of final run time, based on either pb times or golds.
    Pace { golds: bool },
}
