use ron::de::from_reader;
use ron::ser::{to_string_pretty, PrettyConfig};
use sdl2::render::Texture;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;

// The struct that contains data about a speedrun.
// More fields will be added for other split time comparisons, like average and worst times.
#[derive(Debug, Deserialize, Serialize)]
pub struct Run {
    pub game_title: String,
    pub category: String,
    pub offset: Option<u128>,
    pub pb: u128,
    pub splits: Vec<String>,
    best_times: Vec<u128>,
}

impl Run {
    // parse a RON file into a run. Real error handling will come... eventually
    pub fn from_file(filename: &str) -> Self {
        let file = OpenOptions::new().read(true).open(filename).unwrap();
        let run: Self = from_reader(&file).unwrap();
        return run;
    }
    // create an empty run with default values. could implement `Default` but meh
    pub fn new() -> Self {
        Self {
            game_title: "".to_string(),
            category: "".to_string(),
            offset: None,
            pb: 0,
            splits: Vec::new(),
            best_times: Vec::new(),
        }
    }
    // save a run struct to a file (also will get error handling eventually)
    pub fn save(&self, filename: &str) {
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(filename)
            .unwrap();
        let string = to_string_pretty(self, PrettyConfig::new()).unwrap();
        file.write(&string.as_bytes()).unwrap();
    }
    pub fn set_times(&mut self, splits: &Vec<u128>) {
	self.best_times = splits.to_vec();
    }
    pub fn get_times(&self) -> &Vec<u128> {
	&self.best_times
    }
}

pub struct Split<'a> {
    pb_time: u128,
    name_texture: Texture<'a>,
    pb_texture: Texture<'a>,
    current_texture: Option<Texture<'a>>,
}

impl<'a> Split<'a> {
    pub fn new(
        pb_time: u128,
        name_texture: Texture<'a>,
        pb_texture: Texture<'a>,
        current_texture: Option<Texture<'a>>,
    ) -> Self {
        Self {
            pb_time,
            name_texture,
            pb_texture,
            current_texture,
        }
    }
    pub fn time(&self) -> u128 {
        self.pb_time
    }
    pub fn name(&self) -> &Texture {
        &self.name_texture
    }
    pub fn pb(&self) -> &Texture {
        &self.pb_texture
    }
    pub fn cur(&self) -> &Option<Texture> {
        &self.current_texture
    }
    pub fn set_cur(&mut self, cur: Option<Texture<'a>>) {
        self.current_texture = cur;
    }
}
