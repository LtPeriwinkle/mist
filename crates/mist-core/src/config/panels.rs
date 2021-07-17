use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
/// Different types of information panels.
///
/// `golds` field represents whether to compare against gold times rather than pb times.
pub enum Panel {
    SumOfBest,
    CurrentSplitDiff {
        golds: bool,
    },
    Pace {
        golds: bool
    }
}
