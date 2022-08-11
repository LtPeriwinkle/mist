use serde::{Deserialize, Serialize};

/// What a run is comparing to.
#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Comparison {
    Average,
    PersonalBest,
    Golds,
    None,
}

impl Comparison {
    /// Switch to the next comparison.
    pub fn next(&mut self) {
        match self {
            Comparison::Average => {
                *self = Comparison::PersonalBest;
            }
            Comparison::PersonalBest => {
                *self = Comparison::Golds;
            }
            Comparison::Golds => {
                *self = Comparison::None;
            }
            Comparison::None => {
                *self = Comparison::Average;
            }
        }
    }

    /// Switch to the previous comparison.
    pub fn prev(&mut self) {
        match self {
            Comparison::Average => {
                *self = Comparison::None;
            }
            Comparison::PersonalBest => {
                *self = Comparison::Average;
            }
            Comparison::Golds => {
                *self = Comparison::PersonalBest;
            }
            Comparison::None => {
                *self = Comparison::Golds;
            }
        }
    }
}
