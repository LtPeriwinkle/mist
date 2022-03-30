use crate::timer::Run;
use quick_xml::{events::Event, Reader};
use std::fs::File;
use std::io::BufReader;

fn str_to_ms(tm: &str) -> u128 {
    if tm == "00:00:00" || tm.is_empty() {
        0
    } else {
        let hr = &tm[0..2].parse::<u128>().unwrap();
        let min = &tm[3..5].parse::<u128>().unwrap();
        let sec = &tm[6..8].parse::<u128>().unwrap();
        let ms = &tm[9..12].parse::<u128>().unwrap();
        ms + (sec * 1000) + (min * 60000) + (hr * 3600000)
    }
}

/// Constructs a [`Run`] from a LiveSplit split file.
///
/// Attempts to retrieve the relevant information from LiveSplit's XML-based split file
/// in order to construct a mist [`Run`]. Any info that cannot be retrieved is zeroed.
pub struct LssParser {
    filename: String,
}

impl LssParser {
    /// Create a new [`LssParser`] from a [`BufReader`].
    pub fn new(filename: String) -> Self {
        Self { filename }
    }

    /// Retrieve the information from the reader to create a [`Run`].
    ///
    /// Returns a [`Run`] with all of the fields that were found filled in. This can return an empty [`Run`] if the LiveSplit file
    /// was malformed or missing information.
    pub fn parse(&mut self) -> Result<Run, String> {
        let mut run = Run::empty();

        let f = File::open(&self.filename).map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(f);
        let mut reader = Reader::from_reader(&mut reader);
        reader.check_end_names(false);

        let mut buffer = vec![];
        let mut buffer2 = vec![];
        let mut time_str = String::new();
        let mut splits = vec![];
        let mut pb_times = vec![];
        let mut gold_times = vec![];
        let mut sum_times = vec![];
        let mut segment_sum: (u128, u128) = (0, 0);
        let mut pb = 0;

        loop {
            match reader.read_event(&mut buffer) {
                Ok(Event::Start(ref e)) => match e.name() {
                    b"GameName" => {
                        run.set_game_title(
                            reader
                                .read_text(b"GameName", &mut buffer2)
                                .unwrap_or_else(|_| "".to_owned()),
                        );
                    }
                    b"CategoryName" => {
                        run.set_category(
                            reader
                                .read_text(b"CategoryName", &mut buffer2)
                                .unwrap_or_else(|_| "".to_owned()),
                        );
                    }
                    b"Offset" => {
                        let mut off_str = reader
                            .read_text(b"Offset", &mut buffer2)
                            .unwrap_or_else(|_| "".to_owned());
                        off_str.remove(0);
                        let t = str_to_ms(&off_str);
                        run.set_offset(t.into());
                    }
                    b"Name" => {
                        splits.push(
                            reader
                                .read_text(b"Name", &mut buffer2)
                                .unwrap_or_else(|_| "".to_owned()),
                        );
                    }
                    b"RealTime" => {
                        time_str = reader
                            .read_text(b"RealTime", &mut buffer2)
                            .unwrap_or_else(|_| "".to_owned());
                    }
                    b"SegmentHistory" => {
                        segment_sum = (0, 0);
                    }
                    _ => {}
                },
                Ok(Event::End(ref e)) => match e.name() {
                    b"SplitTime" => {
                        let t = str_to_ms(&time_str);
                        if t != 0 {
                            pb_times.push(t.into());
                            pb += t;
                        }
                    }
                    b"BestSegmentTime" => {
                        let t = str_to_ms(&time_str);
                        if t != 0 {
                            gold_times.push(t.into())
                        }
                    }
                    b"Time" => {
                        let t = str_to_ms(&time_str);
                        if t != 0 {
                            segment_sum.0 += 1;
                            segment_sum.1 += t;
                        }
                    }
                    b"SegmentHistory" => {
                        sum_times.push((segment_sum.0, segment_sum.1.into()));
                    }
                    _ => {}
                },
                Ok(Event::Eof) => break,
                _ => {}
            }
        }
        run.set_gold_times(&gold_times);
        run.set_pb_times(&pb_times);
        run.set_sum_times(&sum_times);
        run.set_splits(&splits);
        run.set_pb(pb.into());
        Ok(super::sanify_run(&run))
    }
}
