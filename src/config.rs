// handle configuration of color and font path
use ron::de::from_reader;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;

// more will be added to this in the future
#[derive(Serialize, Deserialize)]
pub struct Config {
    def_file: Option<String>,
    colors: [(u8, u8, u8); 5],
    font_path: String,
}

impl Config {
    pub fn from_file(path: Option<&str>) -> Self {
        let file: std::fs::File;
        match path {
            Some(x) => {
                file = OpenOptions::new()
                    .read(true)
                    .open(x)
                    .expect("file open failed");
            }
            None => {
                file = OpenOptions::new()
                    .read(true)
                    .open("assets/default.mts")
                    .expect("file open failed");
            }
        }
        let cfg: Self = from_reader(&file).unwrap_or(Config::default());
        return cfg;
    }
    pub fn file(&self) -> Option<&String> {
	self.def_file.as_ref()
    }
    pub fn set_file(&mut self, file: &String) {
	self.def_file = Some(file.to_owned());
    }
}

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
            ],
            font_path: "assets/segoe-ui-bold.ttf".to_owned(),
        }
    }
}
