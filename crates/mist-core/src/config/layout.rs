use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct LayoutOpts {
    pub inline_splits: bool,
    pub panels_top: bool,
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
