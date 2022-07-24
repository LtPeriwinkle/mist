use crate::timer::Run;
use ron::{
    de::from_str,
    ser::{to_writer_pretty, PrettyConfig},
};
use serde::Deserialize;
use std::{
    fs::File,
    io::{BufRead, BufReader, Read, Write},
    str::FromStr,
};

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

#[derive(Deserialize)]
struct RunV1 {
    game_title: String,
    category: String,
    offset: Option<u128>,
    pb: u128,
    splits: Vec<String>,
    pb_times: Vec<u128>,
    gold_times: Vec<u128>,
    sum_times: Vec<(u128, u128)>,
}

impl From<LegacyRun> for Run {
    fn from(r: LegacyRun) -> Run {
        Run::new(
            r.category,
            r.game_title,
            r.offset.into(),
            r.pb.into(),
            &r.splits,
            &r.pb_times.iter().map(|&t| t.into()).collect::<Vec<_>>(),
            &r.gold_times.iter().map(|&t| t.into()).collect::<Vec<_>>(),
            &r.pb_times
                .iter()
                .map(|&t| (1u128, t.into()))
                .collect::<Vec<_>>(),
        )
    }
}

impl From<RunV1> for Run {
    fn from(r: RunV1) -> Run {
        Run::new(
            r.category,
            r.game_title,
            r.offset.into(),
            r.pb.into(),
            &r.splits,
            &r.pb_times.iter().map(|&t| t.into()).collect::<Vec<_>>(),
            &r.gold_times.iter().map(|&t| t.into()).collect::<Vec<_>>(),
            &r.sum_times
                .iter()
                .map(|&(n, t)| (n, t.into()))
                .collect::<Vec<_>>(),
        )
    }
}

/// Parses the version and [`Run`] from a mist split file (msf).
pub struct MsfParser {
    filename: String,
}

impl MsfParser {
    /// Create a new [`MsfParser`].
    pub fn new<S: ToString>(filename: S) -> Self {
        Self {
            filename: filename.to_string(),
        }
    }

    /// Attempt to parse a [`Run`] from the file stored in the [`MsfParser`]
    ///
    /// If the file does not specify version in the first line, it is assumed to be a legacy (i.e. not up to date) run
    /// and is treated as such. Runs converted from legacy runs will have the new field(s) filled but zeroed.
    ///
    /// # Errors
    ///
    /// * If the file cannot be read from or is empty.
    /// * If a [`Run`] (legacy or otherwise) cannot be parsed from the file.
    pub fn parse(&self) -> Result<Run, String> {
        let f = File::open(&self.filename).map_err(|e| e.to_string())?;
        self.parse_impl(BufReader::new(f))
    }

    fn parse_impl<R: Read>(&self, reader: BufReader<R>) -> Result<Run, String> {
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
                s.push('\n');
            }
            s
        };
        let run = match version {
            1 => from_str::<RunV1>(&data).map_err(|e| e.to_string())?.into(),
            2 => from_str::<Run>(&data).map_err(|e| e.to_string())?,
            _ => from_str::<LegacyRun>(&data)
                .map_err(|e| e.to_string())?
                .into(),
        };
        Ok(super::sanify_run(&run))
    }

    /// Write the given run to the file stored in the [`MsfParser`].
    pub fn write(&mut self, run: &Run) -> Result<(), String> {
        let run = super::sanify_run(run);
        let mut file = File::create(&self.filename).map_err(|e| e.to_string())?;
        file.write(b"version 2\n").map_err(|e| e.to_string())?;
        to_writer_pretty(&mut file, &run, PrettyConfig::new()).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Set the filename to write the run to.
    pub fn set_filename<S: ToString>(&mut self, new: S) {
        self.filename = new.to_string();
    }

    /// Check whether there is a path stored in the parser or not.
    pub fn no_path(&self) -> bool {
        self.filename.is_empty()
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timer::TimeType::{self, *};

    const V2_RUN: &[u8] = b"version 2\n
        (
            game_title: \"test\",
            category: \"test\",
            offset: Time(200),
            pb: Time(1234),
            splits: [\"test\"],
            pb_times: [Skipped(1234)],
            gold_times: [Time(1234)],
            sum_times: [(2, None)],
        )";

    #[test]
    fn test_parse_v2() {
        let reader = std::io::BufReader::new(V2_RUN);
        let parser = MsfParser::new(String::new());
        let run = parser.parse_impl(reader).unwrap();
        assert_eq!(
            run,
            Run::new(
                "test",
                "test",
                Time(200),
                Time(1234),
                &["test".into()],
                &[Skipped(1234)],
                &[Time(1234)],
                &[(2, TimeType::None)]
            )
        );
    }

    const V1_RUN: &[u8] = b"version 1\n
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
    fn test_parse_v1() {
        let reader = std::io::BufReader::new(V1_RUN);
        let parser = MsfParser::new(String::new());
        let run = parser.parse_impl(reader).unwrap();
        assert_eq!(
            run,
            Run::new(
                "test",
                "test",
                Time(200),
                Time(1234),
                &["test".into()],
                &[Time(1234)],
                &[Time(1234)],
                &[(2, Time(2480))]
            )
        );
    }

    const LEGACY_RUN: &[u8] = b"(
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
        let reader = std::io::BufReader::new(LEGACY_RUN);
        let parser = MsfParser::new(String::new());
        let run = parser.parse_impl(reader).unwrap();
        assert_eq!(
            run,
            Run::new(
                "test",
                "test",
                Time(200),
                Time(1234),
                &["test".into()],
                &[Time(1234)],
                &[Time(1234)],
                &[(1, Time(1234))]
            )
        );
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
            sum_times: [(2, 1234), (1, 1243), (5, 420)],
        )";

    #[test]
    fn test_sanity_check() {
        let reader = std::io::BufReader::new(INSANE_RUN);
        let parser = MsfParser::new(String::new());
        let run = parser.parse_impl(reader).unwrap();
        let run = crate::parse::sanify_run(&run);
        assert_eq!(
            run,
            Run::new(
                "test",
                "test",
                Time(200),
                Time(1234),
                &["test".into(), "test2".into()],
                &[Time(1234), TimeType::None],
                &[Time(1234), TimeType::None],
                &[(2, Time(1234)), (1, Time(1243))]
            )
        );
    }
}
