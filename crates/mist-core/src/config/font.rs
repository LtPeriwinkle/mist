use serde::{Serialize, Deserialize};
use rust_fontconfig::{FcFontCache, FcPattern};
use std::fs::File;
use std::io::Read;

#[derive(Serialize, Deserialize)]
/// A font as represented in the config file
pub struct Font {
    /// whether the font is a system font or in a local file
    system: bool,
    /// the path to the font file or the name of the font
    path_name: String,
}

impl Font {
    pub fn get_bytes(self, buf: &mut [u8]) -> Result<(), String> {
        if !self.system {
            let mut file = File::open(self.path_name).map_err(|e| format!("font file: {}", e))?;
            file.read(buf).map_err(|e| format!("font read: {}", e))?;
            Ok(())
        } else {
            let cache = FcFontCache::build();
            let pat = FcPattern {
                name: Some(self.path_name),
                    .. Default::default()
            };
            let res = cache.query(&pat);
            if let Some(font) = res {
                let mut file = File::open(&font.path).map_err(|e| format!("font file: {}", e))?;
                file.read(buf).map_err(|e| format!("font read: {}", e))?;
            }
            Ok(())
        }
    }
    pub fn timer_default() -> Self {
        Font {
            system: false,
            path_name: "assets/DejaVuSans-Bold.ttf".to_owned()
        }
    }
    pub fn splits_default() -> Self {
        Font {
            system: false,
            path_name: "assets/DejaVuSans.ttf".to_owned()
        }
    }
}

