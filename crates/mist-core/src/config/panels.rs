use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
/// Different types of information panels.
///
/// `golds` field represents whether to compare against gold times rather than pb times.
pub enum Panel {
    SumOfBest,
    CurrentSplitDiff { golds: bool },
    Pace { golds: bool },
}
