use serde::{Serialize, Deserialize};
use rust_fontconfig::{FcFontCache, FcPattern};
use std::fs::File;
use std::io::Read;

#[derive(Serialize, Deserialize)]
/// A font as represented in the config file.
pub struct Font {
    /// Whether the font is a system font or in a local file.
    system: bool,
    /// The path to the font file or the name of the font.
    path_name: String,
}

impl Font {
    /// Get the contents of the font file requested and write them to `buf`.
    ///
    /// # Errors
    /// * If the font cannot be opened, found or read.
    pub fn get_bytes(self, buf: &mut [u8]) -> Result<(), String> {
        if !self.system {
            let mut file = File::open(self.path_name).map_err(|e| format!("font file: {}", e))?;
            file.read(buf).map_err(|e| format!("font read: {}", e))?;
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
            } else {
                return Err("Could not find system font".to_owned());
            }
        }
        Ok(())
    }
    /// Get the default timer font.
    pub fn timer_default() -> Self {
        Font {
            system: false,
            path_name: "assets/DejaVuSans-Bold.ttf".to_owned()
        }
    }
    /// Get the default splits font.
    pub fn splits_default() -> Self {
        Font {
            system: false,
            path_name: "assets/DejaVuSans.ttf".to_owned()
        }
    }
}

