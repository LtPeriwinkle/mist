// handle configuration of color and font path
use ron::de::from_reader;
use ron::ser::{to_string_pretty, PrettyConfig};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use super::KeybindsRaw;
use super::LayoutOpts;
use super::Panel;
#[derive(Serialize, Deserialize)]
#[cfg(feature = "bg")]
/// Configuration of mist.
pub struct Config {
    def_file: Option<String>,
    img_file: Option<String>,
    img_scaled: bool,
    colors: [(u8, u8, u8); 6],
    frame_rounding: u128,
    layout: LayoutOpts,
    panels: Vec<Panel>,
    t_font: String,
    s_font: String,
    font_size: (u16, u16),
    binds: KeybindsRaw,
}

#[derive(Serialize, Deserialize)]
#[cfg(not(feature = "bg"))]
/// Configuration of mist.
pub struct Config {
    def_file: Option<String>,
    colors: [(u8, u8, u8); 6],
    frame_rounding: u128,
    layout: LayoutOpts,
    panels: Vec<Panel>,
    t_font: String,
    s_font: String,
    font_size: (u16, u16),
    binds: KeybindsRaw,
}

impl Config {
    /// Attempts to open the configuration file used by mist and reaturns the Config within if possible.
    ///
    /// If a Config cannot be parsed, returns the default.
    /// Only will return `Err` if it cannot read/write to the config file.
    pub fn open() -> Result<Self, String> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("assets/mist.cfg")
            .map_err(|e| e.to_string())?;
        let cfg: Self = from_reader(&file).unwrap_or(Config::default());
        return Ok(cfg);
    }
    /// Get the split file from the Config. Returns None if no file set.
    pub fn file(&self) -> Option<&String> {
        self.def_file.as_ref()
    }
    #[cfg(feature = "bg")]
    /// Get the path to the image file to be used as a background for the timer.
    pub fn img(&self) -> Option<&String> {
        self.img_file.as_ref()
    }
    #[cfg(feature = "bg")]
    /// Determine whether the image should be scaled to fit the screen or cropped.
    pub fn img_scaled(&self) -> bool {
        self.img_scaled
    }
    /// Set the split file path to a new one.
    pub fn set_file(&mut self, file: &String) {
        self.def_file = Some(file.to_owned());
    }
    /// Get the path to the font used for the display timer.
    pub fn tfont(&self) -> &str {
        &self.t_font
    }
    /// Get the path to the font used for the rows of splits.
    pub fn sfont(&self) -> &str {
        &self.s_font
    }
    /// Get the tuple of font sizes for the timer and split fonts respectively.
    pub fn fsize(&self) -> (u16, u16) {
        self.font_size
    }
    /// Get the list of colors to be used for the timer.
    pub fn color_list(&self) -> [(u8, u8, u8); 6] {
        self.colors
    }
    /// Write the config to the file.
    ///
    /// # Errors
    ///
    /// * If the serialization fails
    /// * If the file cannot be written to or opened.
    pub fn save(&self) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .write(true)
            .open("assets/mist.cfg")
            .map_err(|e| e.to_string())?;
        let string = to_string_pretty(self, PrettyConfig::new()).map_err(|e| e.to_string())?;
        file.write(&string.as_bytes()).map_err(|e| e.to_string())?;
        Ok(())
    }
    /// Get the keybinds in string form as names of keys
    pub fn binds(&self) -> &KeybindsRaw {
        &self.binds
    }
    /// Get the layout options
    pub fn layout(&self) -> &LayoutOpts {
        &self.layout
    }
    /// Get the list of timing display panels
    pub fn panels(&self) -> &Vec<Panel> {
        &self.panels
    }
    pub fn rounding(&self) -> u128 {
        self.frame_rounding
    }
}

#[cfg(feature = "bg")]
impl Default for Config {
    fn default() -> Config {
        Config {
            def_file: None,
            img_file: None,
            img_scaled: false,
            colors: [
                (0, 255, 0),
                (255, 0, 0),
                (255, 90, 90),
                (135, 255, 125),
                (255, 255, 0),
                (0, 0, 0),
            ],
            frame_rounding: 30,
            layout: LayoutOpts::default(),
            panels: vec![],
            t_font: "assets/segoe-ui-bold.ttf".to_owned(),
            s_font: "assets/segoe-ui-bold.ttf".to_owned(),
            font_size: (60, 25),
            binds: KeybindsRaw::default(),
        }
    }
}
#[cfg(not(feature = "bg"))]
impl Default for Config {
    fn default() -> Config {
        Config {
            def_file: None,
            colors: [
                (0, 255, 0),
                (255, 0, 0),
                (255, 90, 90),
                (135, 255, 125),
                (255, 255, 0),
                (0, 0, 0),
            ],
            frame_rounding: 30,
            layout: LayoutOpts::default(),
            panels: vec![],
            t_font: "assets/segoe-ui-bold.ttf".to_owned(),
            s_font: "assets/segoe-ui-bold.ttf".to_owned(),
            font_size: (60, 25),
            binds: KeybindsRaw::default(),
        }
    }
}
