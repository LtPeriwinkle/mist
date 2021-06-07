use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum Panel {
    SumOfBest,
    CurrentSplitDiff {
        golds: bool,
    },
    Pace {
        golds: bool
    }
}
