use super::Colors;
use super::Font;
use super::KeybindsRaw;
use super::LayoutOpts;
use super::Panel;
use ron::de::from_reader;
use ron::extensions::Extensions;
use ron::ser::{to_string_pretty, PrettyConfig};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Serialize, Deserialize, Debug)]
/// Configuration of mist.
pub struct Config {
    def_file: Option<String>,
    win_size: (u32, u32),
    #[cfg(feature = "bg")]
    img_file: Option<String>,
    #[cfg(feature = "bg")]
    img_scaled: bool,
    colors: Colors,
    frame_rounding: Option<u128>,
    layout: LayoutOpts,
    panels: Vec<Panel>,
    t_font: Font,
    s_font: Font,
    ms_ratio: f32,
    binds: KeybindsRaw,
}

impl Config {
    /// Attempts to open and parse mist's default config.
    ///
    /// If a Config cannot be parsed, returns the default.
    /// Only will return `Err` if it cannot read the config file.
    pub fn open() -> Result<Self, String> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("assets/mist.cfg")
            .map_err(|e| e.to_string())?;
        let cfg: Self = from_reader(&file).unwrap_or_default();
        Ok(cfg)
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
    pub fn set_file(&mut self, file: &str) {
        self.def_file = Some(file.to_owned());
    }
    /// Get the Font used for the display timer.
    pub fn tfont(&self) -> &Font {
        &self.t_font
    }
    /// Get the Font used for the rows of splits.
    pub fn sfont(&self) -> &Font {
        &self.s_font
    }
    /// Get the list of colors to be used for the timer.
    pub fn colors(&self) -> Colors {
        self.colors
    }
    /// Write the config to the file.
    ///
    /// # Errors
    ///
    /// * If the serialization fails.
    /// * If the file cannot be written to or opened.
    pub fn save(&self) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .write(true)
            .open("assets/mist.cfg")
            .map_err(|e| e.to_string())?;
        let string = to_string_pretty(
            self,
            PrettyConfig::new().extensions(Extensions::IMPLICIT_SOME),
        )
        .map_err(|e| e.to_string())?;
        file.write(string.as_bytes()).map_err(|e| e.to_string())?;
        Ok(())
    }
    /// Get the keybinds in string form as names of keys.
    pub fn binds(&self) -> &KeybindsRaw {
        &self.binds
    }
    /// Get the layout options.
    pub fn layout(&self) -> &LayoutOpts {
        &self.layout
    }
    /// Get the list of timing display panels.
    pub fn panels(&self) -> &Vec<Panel> {
        &self.panels
    }
    /// Get the requested framerate to round times to.
    /// None representes no rounding.
    pub fn rounding(&self) -> Option<u128> {
        self.frame_rounding
    }
    /// Get the ratio of millisecond font size to timer font size
    pub fn ms_ratio(&self) -> f32 {
        self.ms_ratio
    }
    pub fn win_size(&self) -> (u32, u32) {
        self.win_size
    }
    pub fn set_win_size(&mut self, new: (u32, u32)) {
        self.win_size = new;
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            def_file: None,
            win_size: (300, 500),
            #[cfg(feature = "bg")]
            img_file: None,
            #[cfg(feature = "bg")]
            img_scaled: false,
            frame_rounding: Some(30),
            colors: Colors::default(),
            layout: LayoutOpts::default(),
            panels: vec![],
            t_font: Font::timer_default(),
            s_font: Font::splits_default(),
            ms_ratio: 1.0,
            binds: KeybindsRaw::default(),
        }
    }
}
