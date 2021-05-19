use crate::run::Run;
use ron::de::from_bytes;
use std::io::BufRead;
use std::io::{Error, ErrorKind};

/// Parses the version and [Run] from a mist split file (msf)
pub struct MsfParser<R: BufRead> {
    reader: R,
    version: u32,
}

impl<R: BufRead> MsfParser<R> {
    const LEGACY: u32 = 0xBA5ED;
    /// Create a new MsfParser with the given reader. Reader must implement [BufRead].
    ///
    /// # Errors
    ///
    /// * If a line cannot be read from the reader.
    pub fn new(mut reader: R) -> Result<Self, Error> {
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
