// livesplit lss file parsing will go here
use crate::run::Run;
use quick_xml::{Reader, events::Event};
use std::fs::{File, OpenOptions};
use std::io::BufReader;

fn str_to_ms(tm: String) -> u128 {
	let hr = &tm[0..2].parse::<u128>().unwrap();
	let min = &tm[3..5].parse::<u128>().unwrap();
	let sec = &tm[6..8].parse::<u128>().unwrap();
	let ms = &tm[9..12].parse::<u128>().unwrap();
	return ms + (sec * 1000) + (min * 60000) + (hr * 3600000);
}

fn parse_times(mut times: Vec<String>) -> Vec<u128> {
    	let mut real_time_vec: Vec<u128> = Vec::new();
        let mut split_time_vec: Vec<u128> = Vec::new();
        let mut index = 0;
    	while times.len() > 0 {
		real_time_vec.push(str_to_ms(times.pop().unwrap()));
        }
        while index < real_time_vec.len() {
		if index != real_time_vec.len() - 1 {            
			split_time_vec.push(real_time_vec[index] - real_time_vec[index + 1]);
		} else {
			split_time_vec.push(real_time_vec[index]);
    		}
    		index += 1;
        }
        split_time_vec.reverse();
        split_time_vec

}

impl Run {
	pub fn from_lss_file(filename: &str) -> Option<Run> {
		let file: File;
		match OpenOptions::new().read(true).open(filename) {
			Ok(x) => {
				file = x;
    			}
    			Err(_) => {
				return None;
        		}
    		}
    		let bufreader = BufReader::new(file);
    		let mut reader = Reader::from_reader(bufreader);
    		let mut buffer = Vec::new();

    		let mut tm_vec: Vec<String> = vec![];
    		let mut gold_vec: Vec<String> = vec![];
    		let mut names: Vec<String> = vec![];
    		let mut title = String::new();
    		let mut cat = String::new();
    		let mut off_str = String::new();
    		let mut string: String;

    		let mut split_time = false;
    		let mut real_time = false;
    		let mut gold = false;
    		loop {
			match reader.read_event(&mut buffer) {
				Ok(Event::Start(ref e)) => {
					match e.name() {
						b"SplitTime" => {
    							split_time = true;
    						}
    						b"Name" => {
    							string = reader.read_text(e.name(), &mut Vec::new()).unwrap_or("".to_string());
        						names.push(string);
        					}
        					b"RealTime" => {
							real_time = true;
            					}
            					b"GameName" => {
    							string = reader.read_text(e.name(), &mut Vec::new()).unwrap_or("".to_string());
							title = string;
                				}
                				b"CategoryName" => {
    							string = reader.read_text(e.name(), &mut Vec::new()).unwrap_or("".to_string());
    							cat = string;
                    				}
                    				b"Offset" => {
    							string = reader.read_text(e.name(), &mut Vec::new()).unwrap_or("".to_string());
    							off_str = string;
                        			}
                        			b"BestSegmentTime" => {
							gold = true;
                            			}
						_ => {}
    					}
    				}
    				Ok(Event::End(ref e)) => {
					match e.name() {
						b"SplitTime" => {
    							split_time = false;
    						}
        					b"RealTime" => {
							real_time = false;
            					}
                        			b"BestSegmentTime" => {
							gold = false;
                            			}
						_ => {}
    					}
        			}
        			Ok(Event::Text(ref e)) => {
            					if real_time {
							if split_time {
                						string = e.unescape_and_decode(&reader).unwrap();
                						tm_vec.push(string);
    							} else if gold {
                						string = e.unescape_and_decode(&reader).unwrap();
                						gold_vec.push(string);
        						}
                				}
            			}
        			Ok(Event::Eof) => break,
        			Err(e) => {panic!("{:?}", e)}
        			_ => {}
    			}
    			buffer.clear()
        	}
        	off_str.remove(0);
        	let pb_splits = parse_times(tm_vec);
        	let gold_splits = gold_vec.iter().map(|val| str_to_ms(val.to_owned())).collect();
        	let off = match str_to_ms(off_str) {
				0 => None,
				y => Some(y),
            		  };
        	let run = Run::new(
			title,
			cat,
			off,
			{let mut pb = 0; for i in &pb_splits {pb += i;} pb},
			names,
			pb_splits,
			gold_splits
        	);
        	println!("{:?}", run);
        	return Some(run);
    	}
}
