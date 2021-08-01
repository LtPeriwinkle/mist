use crate::run::Run;
use ron::de::from_str;
use ron::ser::{to_writer_pretty, PrettyConfig};
use serde::Deserialize;
use std::io::{BufRead, Write};
use std::str::FromStr;

#[derive(Deserialize)]
struct LegacyRun {
    game_title: String,
    category: String,
    offset: Option<u128>,
    pb: u128,
    splits: Vec<String>,
    pb_times: Vec<u128>,
    gold_times: Vec<u128>,
}

impl Into<Run> for LegacyRun {
    fn into(self) -> Run {
        let sums = self.pb_times.iter().map(|val| (1u128, *val)).collect();
        Run::new(
            self.category,
            self.game_title,
            self.offset,
            self.pb,
            &self.splits,
            &self.pb_times,
            &self.gold_times,
            &sums,
        )
    }
}

/// Parses the version and [`Run`] from a mist split file (msf)
pub struct MsfParser {}

impl MsfParser {
    pub const VERSION: u8 = 1;
    /// Create a new MsfParser.
    pub fn new() -> Self {
        MsfParser {}
    }
    /// Attempt to parse a [`Run`] from the given reader. Reader must implement [`BufRead`].
    ///
    /// If the file does not specify version in the first line, it is assumed to be a legacy (i.e. not up to date) run
    /// and is treated as such. Runs converted from legacy runs will have the new field(s) filled but zeroed.
    ///
    /// # Errors
    ///
    /// * If the reader cannot be read from or is empty.
    /// * If a [`Run`] (legacy or otherwise) cannot be parsed from the reader.
    pub fn parse<R: BufRead>(&self, reader: R) -> Result<Run, String> {
        let mut lines = reader.lines().map(|l| l.unwrap());
        // TODO: better error handling
        let ver_info = String::from_str(&lines.next().ok_or("Input was empty.")?).unwrap();
        let version: u32 = match ver_info.rsplit_once(' ') {
            Some(num) => num.1.parse::<u32>().unwrap_or(0),
            None => 0,
        };
        let data = {
            let mut s = String::new();
            if version == 0 {
                s.push_str(&ver_info);
            }
            for line in lines {
                s.push_str(&line);
            }
            s
        };
        let run = self.run_sanity(if version == 0 {
            from_str::<LegacyRun>(&data)
                .map_err(|e| e.to_string())?
                .into()
        } else {
            from_str(&data).map_err(|e| e.to_string())?
        });
        Ok(run)
    }
    /// Write the given run to the given writer.
    pub fn write<W: Write>(&self, run: &Run, mut writer: W) -> Result<(), String> {
        let run = self.run_sanity(run.clone());
        writer.write(b"version 1\n").map_err(|e| e.to_string())?;
        to_writer_pretty(&mut writer, &run, PrettyConfig::new()).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn run_sanity(&self, run: Run) -> Run {
        let mut run = run;
        let len = run.splits().len();
        let mut golds = run.gold_times().to_owned();
        let mut times = run.pb_times().to_owned();
        let mut sums = run.sum_times().to_owned();
        if golds.len() < len {
            golds.resize_with(len, Default::default);
            run.set_gold_times(&golds);
        }
        if times.len() < len {
            times.resize_with(len, Default::default);
            run.set_pb_times(&times);
        }
        if sums.len() < len {
            sums.resize_with(len, Default::default);
            run.set_sum_times(&sums);
        }
        run
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const V1RUN: &[u8] = b"version 1\n
        (
            game_title: \"test\",
            category: \"test\",
            offset: Some(200),
            pb: 1234,
            splits: [\"test\"],
            pb_times: [1234],
            gold_times: [1234],
            sum_times: [(2, 2480)],
        )";
    #[test]
    fn test_parse() {
        let reader = std::io::BufReader::new(V1RUN);
        let parser = MsfParser::new();
        let run = parser.parse(reader);
        println!("{:?}", run);
        assert!(run.is_ok());
    }

    const LEGACYRUN: &[u8] = b"(
        game_title: \"test\",
        category: \"test\",
        offset: Some(200),
        pb: 1234,
        splits: [\"test\"],
        pb_times: [1234],
        gold_times: [1234],
    )";

    #[test]
    fn test_parse_legacy() {
        let reader = std::io::BufReader::new(LEGACYRUN);
        let parser = MsfParser::new();
        let run = parser.parse(reader);
        assert!(run.is_ok());
    }

    const INSANE_RUN: &[u8] = b"version 1\n
        (
            game_title: \"test\",
            category: \"test\",
            offset: Some(200),
            pb: 1234,
            splits: [\"test\", \"test2\"],
            pb_times: [1234],
            gold_times: [1234],
            sum_times: [(2, 1234)],
        )";

    #[test]
    fn test_sanity_check() {
        let reader = std::io::BufReader::new(INSANE_RUN);
        let parser = MsfParser::new();
        let run = parser.parse(reader);
        assert!(run.is_ok());
        let run = run.unwrap();
        assert_eq!(run.gold_times().to_owned(), vec![1234, 0]);
        assert_eq!(run.pb_times().to_owned(), vec![1234, 0]);
        assert_eq!(run.sum_times().to_owned(), vec![(2, 1234), (0, 0)]);
    }
}
