use serde::{Deserialize, Serialize};
use std::ops::{AddAssign, Div, SubAssign};

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize, Debug)]
pub enum TimeType {
    Skipped(u128),
    Time(u128),
    None,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum DiffType {
    Skipped(i128),
    Time(i128),
}

impl TimeType {
    pub fn from_time(time: u128) -> Self {
        if time == 0 {
            Self::None
        } else {
            Self::Time(time)
        }
    }
    pub fn from_option(opt: Option<u128>) -> Self {
        if let Some(x) = opt {
            Self::Time(x)
        } else {
            Self::None
        }
    }
    pub fn val(self) -> u128 {
        if let TimeType::Time(x) = self {
            x
        } else {
            0
        }
    }
    pub fn raw(self) -> u128 {
        if let TimeType::Skipped(x) | TimeType::Time(x) = self {
            x
        } else {
            0
        }
    }
    pub fn to_option(self) -> Option<u128> {
        if let TimeType::Time(x) = self {
            Some(x)
        } else {
            None
        }
    }
    pub fn is_time(self) -> bool {
        matches!(self, TimeType::Time(_))
    }
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
