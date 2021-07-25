use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
/// Options for mist's visual layout.
pub struct LayoutOpts {
    /// Whether to put the split names on the same line as their times.
    pub inline_splits: bool,
    /// Whether to place the information panels at the top of the window.
    pub panels_top: bool,
    /// Whether to place the large display timer at the top of the window.
    pub timer_top: bool,
}

impl Default for LayoutOpts {
    fn default() -> Self {
        LayoutOpts {
            inline_splits: true,
            panels_top: false,
            timer_top: false,
        }
    }
}
