use crate::run::Run;
use ron::de::from_bytes;
use ron::ser::{to_writer_pretty, PrettyConfig};
use std::io::{BufRead, BufReader, Read};
use std::io::{Error, ErrorKind};
use std::fs::File;

/// Parses the version and [Run] from a mist split file (msf)
pub struct MsfParser {
    reader: BufReader<File>,
    version: u32,
}

impl MsfParser {
    const LEGACY: u32 = 0xBA5ED;
    /// Create a new MsfParser with the given reader. Reader must implement [BufRead].
    ///
    /// # Errors
    ///
    /// * If a line cannot be read from the reader.
    pub fn new(mut reader: BufReader<File>) -> Result<Self, Error> {
        let mut ver_info = String::new();
        while ver_info.is_empty() {
            reader.read_line(&mut ver_info)?;
        }
        let version: u32 = match ver_info.rsplit_once(' ') {
            Some(num) => num.1.parse::<u32>().unwrap_or(Self::LEGACY),
            None => Self::LEGACY,
        };
        Ok(MsfParser { reader, version })
    }
    pub fn set_reader(&mut self, reader: BufReader<File>) {
        self.reader = reader
    }
    /// Determine whether the Run will need converting to the current version, i.e. it is legacy or just outdated.
    pub fn needs_converting(&self) -> bool {
        if self.version == Self::LEGACY {
            true
        } else {
            false
        }
    }
    /// Try to get the Run from the reader. Currently, updating runs is not yet supported.
    ///
    /// # Errors
    ///
    /// * If the Run will require converting from an older version (temporary, will be removed eventually)
    /// * If a Run cannot be parsed from the data in the reader
    pub fn parse(&mut self) -> Result<Run, Error> {
        if self.needs_converting() {
            return Err(Error::new(ErrorKind::Other, "legacy parsing not yet implemented"));
        }
        let mut data: Vec<u8> = vec![];
        // TODO: better error handling
        let _ = self.reader.read_to_end(&mut data)?;
        from_bytes(&mut data).map_err(|e| {Error::new(ErrorKind::InvalidData, format!("{}", e))})
    }
}

pub struct MsfWriter {
    writer: File,
    config: PrettyConfig
}

impl MsfWriter {
    pub fn new(writer: File) -> Self {
        MsfWriter {writer, config: PrettyConfig::new().with_enumerate_arrays(true)}
    }
    pub fn write(&mut self, run: Run) -> Result<(), String> {
        to_writer_pretty(&mut self.writer, &run, self.config.clone()).map_err(|e| {e.to_string()})
    }
}
