use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
/// Colors used in the mist interface, as RGBA
pub struct Colors {
    pub ahead: (u8, u8, u8, u8),
    pub behind: (u8, u8, u8, u8),
    pub gaining: (u8, u8, u8, u8),
    pub losing: (u8, u8, u8, u8),
    pub gold: (u8, u8, u8, u8),
    pub highlight: (u8, u8, u8, u8),
    pub line: (u8, u8, u8, u8),
    pub background: (u8, u8, u8, u8),
    pub text: (u8, u8, u8, u8),
}

impl Default for Colors {
    fn default() -> Colors {
        Colors {
            ahead: (0, 255, 0, 255),
            behind: (255, 0, 0, 255),
            gaining: (255, 90, 90, 255),
            losing: (135, 255, 125, 255),
            gold: (255, 255, 0, 255),
            line: (128, 128, 128, 255),
            highlight: (0, 0, 255, 255),
            background: (0, 0, 0, 0),
            text: (255, 255, 255, 255),
        }
    }
}
