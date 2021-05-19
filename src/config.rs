// handle configuration of color and font path
use ron::de::from_reader;
use ron::ser::{to_string_pretty, PrettyConfig};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
// more will be added to this in the future
#[derive(Serialize, Deserialize)]
#[cfg(feature = "bg")]
pub struct Config {
    def_file: Option<String>,
    img_file: Option<String>,
    img_scaled: bool,
    colors: [(u8, u8, u8); 6],
    t_font: String,
    s_font: String,
    font_size: (u16, u16),
}

#[derive(Serialize, Deserialize)]
#[cfg(not(feature = "bg"))]
pub struct Config {
    def_file: Option<String>,
    colors: [(u8, u8, u8); 6],
    t_font: String,
    s_font: String,
    font_size: (u16, u16),
}

impl Config {
    // open the configuration file in the assets directory
    // if the file does not exist, then creates it and if any error occurs then returns the default configuration
    pub fn open() -> Result<Self, String> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("assets/mist.cfg")
            .map_err(|e| {e.to_string()})?;
        let cfg: Self = from_reader(&file).unwrap_or(Config::default());
        return Ok(cfg);
    }
    pub fn file(&self) -> Option<&String> {
        self.def_file.as_ref()
    }
    #[cfg(feature = "bg")]
    pub fn img(&self) -> Option<&String> {
        self.img_file.as_ref()
    }
    #[cfg(feature = "bg")]
    pub fn img_scaled(&self) -> bool {
        self.img_scaled
    }
    pub fn set_file(&mut self, file: &String) {
        self.def_file = Some(file.to_owned());
    }
    pub fn tfont(&self) -> &str {
        &self.t_font
    }
    pub fn sfont(&self) -> &str {
        &self.s_font
    }
    pub fn fsize(&self) -> (u16, u16) {
        self.font_size
    }
    pub fn color_list(&self) -> [(u8, u8, u8); 6] {
        self.colors
    }
    pub fn save(&self) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .write(true)
            .open("assets/mist.cfg")
            .map_err(|e| {e.to_string()})?;
        let string = to_string_pretty(self, PrettyConfig::new()).unwrap();
        file.write(&string.as_bytes()).map_err(|e| {e.to_string()})?;
        Ok(())
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
            t_font: "assets/segoe-ui-bold.ttf".to_owned(),
            s_font: "assets/segoe-ui-bold.ttf".to_owned(),
            font_size: (60, 25),
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
            t_font: "assets/segoe-ui-bold.ttf".to_owned(),
            s_font: "assets/segoe-ui-bold.ttf".to_owned(),
            font_size: (60, 25),
        }
    }
}
