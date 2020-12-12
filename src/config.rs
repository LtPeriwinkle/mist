// handle configuration of color and font path
use std::fs::OpenOptions;
use ron::de::from_reader;
use sdl2::pixels::Color;

// more will be added to this in the future
pub struct Config<'a> {
	def_file: Option<String>,
	colors: [(u8, u8, u8); 4],
	font_path: &'a str
}

impl<'a> Config<'a> {
	pub fn from_file(path: Option<&str>) -> Option<Self> {
    		let file: std::fs::File;
    		match path {
			Some(x) => {
    				file = OpenOptions::new().read(true).open(x).expect("file open failed");
			}
			None => {
				file = OpenOptions::new().read(true).open("assets/default.mts").expect("file open failed");
			}
    		}
    		let cfg: Self = from_reader(&file);
		return cfg.ok();
	}
}
