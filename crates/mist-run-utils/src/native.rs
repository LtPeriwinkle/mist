// msf split file parsing and Run utilities
use crate::run::Run;
use ron::de::from_reader;
use std::fs::OpenOptions;
use ron::ser::{PrettyConfig, to_string_pretty};
use std::io::Write;

impl Run {
    /// Create a run from an MSF (aka mist split file). If the file is malformed or a file error occurs, return None.
    /// Creates the file if it does not exist so it can be saved to when the timer is closed.
    pub fn from_msf_file(filename: &str) -> Option<Run> {
        let file: std::fs::File;
        match OpenOptions::new().read(true).write(true).create(true).open(filename) {
            Ok(x) => {
                file = x;
            }
            Err(_) => {
                return None;
            }
        }
        let run = from_reader(&file);
        run.ok()
    }
    /// save the run to an msf file provided. no error handling yet unfortunately
    /// creates the file if it does not exist
    pub fn save_msf(&self, filename: &str) {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(filename)
            .unwrap();
        let string = to_string_pretty(self, PrettyConfig::new()).unwrap();
        file.write_all(&string.as_bytes()).unwrap();
    }
}
