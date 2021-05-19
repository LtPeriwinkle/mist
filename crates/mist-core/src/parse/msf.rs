use crate::run::Run;
use ron::de::from_bytes;
use ron::ser::{to_writer_pretty, PrettyConfig};
use std::io::{BufRead, Write};

/// Parses the version and [Run] from a mist split file (msf)
pub struct MsfParser {}

impl MsfParser {
    /// Create a new MsfParser.
    pub fn new() -> Self {
        MsfParser {}
    }
    /// Attempt to parse a [Run] from the given reader. Reader must implement [BufRead].
    ///
    /// # Errors
    ///
    /// * If the reader cannot be read from.
    /// * If the run found is a legacy run (for now).
    /// * If a Run cannot be parsed from the reader.
    pub fn parse<R: BufRead>(&self, mut reader: R) -> Result<Run, String> {
        let mut ver_info = String::new();
        while ver_info.is_empty() {
            reader.read_line(&mut ver_info).map_err(|e| {e.to_string()})?;
        }
        let _version: u32 = match ver_info.rsplit_once(' ') {
            Some(num) => num.1.parse::<u32>().unwrap_or(0),
            None => return Err("legacy parsing not yet implemented".to_owned()),
        };
            
        let mut data: Vec<u8> = vec![];
        // TODO: better error handling
        let _ = reader.read_to_end(&mut data).map_err(|e| {e.to_string()})?;
        from_bytes(&mut data).map_err(|e| {e.to_string()})
    }
    /// Write the given run to the given writer.
    pub fn write<W: Write>(&self, run: &Run, mut writer: W) -> Result<(), String> {
        to_writer_pretty(&mut writer, run, PrettyConfig::new()).map_err(|e| {e.to_string()})?;
        Ok(())
    }
}
