use rust_fontconfig::{FcFontCache, FcPattern};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
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
    pub fn get_path(&self) -> Result<String, String> {
        if !self.system {
            Ok(self.path_name.clone())
        } else {
            let cache = FcFontCache::build();
            let pat = FcPattern {
                name: Some(self.path_name.clone()),
                ..Default::default()
            };
            let res = cache.query(&pat);
            if let Some(font) = res {
                return Ok(font.path.clone());
            } else {
                return Err("Could not find system font".to_owned());
            }
        }
    }
    /// Get the default timer font.
    pub fn timer_default() -> Self {
        Font {
            system: false,
            path_name: "assets/DejaVuSans-Bold.ttf".to_owned(),
        }
    }
    /// Get the default splits font.
    pub fn splits_default() -> Self {
        Font {
            system: false,
            path_name: "assets/DejaVuSans.ttf".to_owned(),
        }
    }
}
