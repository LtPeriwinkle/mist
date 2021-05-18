use crate::run::Run;
use quick_xml::{events::Event, Reader};
use std::io::BufRead;

fn str_to_ms(tm: &String) -> u128 {
    if tm == "00:00:00" || tm == "" {
        return 0;
    }
    let hr = &tm[0..2].parse::<u128>().unwrap();
    let min = &tm[3..5].parse::<u128>().unwrap();
    let sec = &tm[6..8].parse::<u128>().unwrap();
    let ms = &tm[9..12].parse::<u128>().unwrap();
    return ms + (sec * 1000) + (min * 60000) + (hr * 3600000);
}

pub struct LssParser<R: BufRead> {
    reader: R,
}

impl<R: BufRead> LssParser<R> {
    pub fn new(reader: R) -> Self {
        LssParser { reader }
    }
    pub fn parse(&mut self) -> Run {
        let mut run = Run::empty();

        let mut reader = Reader::from_reader(&mut self.reader);
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
                                .unwrap_or("".to_owned()),
                        );
                    }
                    b"CategoryName" => {
                        run.set_category(
                            reader
                                .read_text(b"CategoryName", &mut buffer2)
                                .unwrap_or("".to_owned()),
                        );
                    }
                    b"Offset" => {
                        let mut off_str = reader
                            .read_text(b"Offset", &mut buffer2)
                            .unwrap_or("".to_owned());
                        off_str.remove(0);
                        match str_to_ms(&off_str) {
                            0 => {}
                            t => run.set_offset(Some(t)),
                        }
                    }
                    b"Name" => {
                        splits.push(
                            reader
                                .read_text(b"Name", &mut buffer2)
                                .unwrap_or("".to_owned()),
                        );
                    }
                    b"RealTime" => {
                        time_str = reader
                            .read_text(b"RealTime", &mut buffer2)
                            .unwrap_or("".to_owned());
                    }
                    b"SegmentHistory" => {
                        segment_sum = (0, 0);
                    }
                    _ => {}
                },
                Ok(Event::End(ref e)) => match e.name() {
                    b"SplitTime" => match str_to_ms(&time_str) {
                        0 => {}
                        t => {
                            pb_times.push(t);
                            pb += t;
                        }
                    },
                    b"BestSegmentTime" => match str_to_ms(&time_str) {
                        0 => {}
                        t => {
                            gold_times.push(t);
                        }
                    },
                    b"Time" => {
                        segment_sum.0 += 1;
                        segment_sum.1 += str_to_ms(&time_str);
                    }
                    b"SegmentHistory" => {
                        sum_times.push(segment_sum);
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
        run.set_pb(pb);
        run
    }
}
