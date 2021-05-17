use crate::run::Run;
use std::io::BufRead;
use std::io::Error;

pub struct MsfParser<R: BufRead> {
    reader: R,
    version: u32,
}

impl<R: BufRead> MsfParser<R> {
    const LEGACY: u32 = 0xBA5ED;
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
    pub fn needs_converting(&self) -> bool {
        if self.version == Self::LEGACY {
            true
        } else {
            false
        }
    }
}
