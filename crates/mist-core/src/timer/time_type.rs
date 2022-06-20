use serde::{Deserialize, Serialize};
use std::ops::{AddAssign, Div, SubAssign};

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize, Debug)]
/// A type of time recorded with the timer.
pub enum TimeType {
    /// The split was skipped, but the time that was taken on it is
    /// still recorded, for calculating comparisons.
    Skipped(u128),
    /// A regular time.
    Time(u128),
    /// No time, can be used in a split file to signal that a split or time
    /// has never been completed.
    None,
}

#[derive(PartialEq, Copy, Clone, Debug)]
/// A type of difference from a previous split.
pub enum DiffType {
    /// The split was skipped, but the time difference is still
    /// recorded, for calculating comparisons.
    Skipped(i128),
    /// A regular diff.
    Time(i128),
}

impl TimeType {
    /// Convert from a `u128` to `TimeType`.
    /// If `time` is 0, returns `None`, otherwise returns
    /// `Time(time)`
    pub fn from_time(time: u128) -> Self {
        if time == 0 {
            Self::None
        } else {
            Self::Time(time)
        }
    }
    /// Convert from `Option<u128>` to `TimeType`.
    /// If `opt` is `None`, returns `None`, otherwise
    /// retuns `Time` holding the value within `opt`.
    pub fn from_option(opt: Option<u128>) -> Self {
        if let Some(x) = opt {
            Self::Time(x)
        } else {
            Self::None
        }
    }
    /// Convert from `TimeType` to `u128`.
    /// If `self` is not `Time`, returns 0, otherwise returns the contained
    /// value.
    pub fn val(self) -> u128 {
        if let TimeType::Time(x) = self {
            x
        } else {
            0
        }
    }
    /// Convert from `TimeType` to `u128`.
    /// If `self` is `None`, returns 0, otherwise returns the contained value.
    pub fn raw(self) -> u128 {
        if let TimeType::Skipped(x) | TimeType::Time(x) = self {
            x
        } else {
            0
        }
    }
    /// Convert from `TimeType` to `u128`.
    /// If `self` is not `Time`, returns `None`.
    pub fn to_option(self) -> Option<u128> {
        if let TimeType::Time(x) = self {
            Some(x)
        } else {
            None
        }
    }
    /// Returns whether `self` is `Time`.
    pub fn is_time(self) -> bool {
        matches!(self, TimeType::Time(_))
    }
    /// Returns whether `self` is `None`.
    pub fn is_none(self) -> bool {
        self == TimeType::None
    }
}

impl From<u128> for TimeType {
    fn from(t: u128) -> TimeType {
        if t == 0 {
            Self::None
        } else {
            Self::Time(t)
        }
    }
}

impl From<Option<u128>> for TimeType {
    fn from(opt: Option<u128>) -> TimeType {
        if let Some(x) = opt {
            x.into()
        } else {
            Self::None
        }
    }
}

impl DiffType {
    /// Convert from `DiffType` to `i128`.
    /// Returns the inner value whether `self` is `Skipped` or `Time`.
    pub fn raw(self) -> i128 {
        let (DiffType::Skipped(x) | DiffType::Time(x)) = self;
        x
    }
}

impl SubAssign<u128> for TimeType {
    fn sub_assign(&mut self, other: u128) {
        match *self {
            TimeType::Skipped(x) => *self = TimeType::Skipped(x - other),
            TimeType::Time(x) => *self = TimeType::Time(x - other),
            _ => {} // just don't subtract nones :)
        }
    }
}

impl AddAssign<u128> for TimeType {
    fn add_assign(&mut self, other: u128) {
        match *self {
            TimeType::Skipped(x) => *self = TimeType::Skipped(x + other),
            TimeType::Time(x) => *self = TimeType::Time(x + other),
            TimeType::None => *self = TimeType::Time(other),
        }
    }
}

impl Div<u128> for TimeType {
    type Output = u128;

    fn div(self, rhs: u128) -> u128 {
        if let TimeType::Skipped(x) | TimeType::Time(x) = self {
            x / rhs
        } else {
            0
        }
    }
}

impl Default for TimeType {
    fn default() -> Self {
        Self::None
    }
}
