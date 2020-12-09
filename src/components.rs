// miscellaneous stuff that doesnt really fit anywhere else
use sdl2::pixels::Color;
use std::env;
use tinyfiledialogs as tfd;

pub const SPLITS_ON_SCREEN: usize = 8; // max splits allowed on screen
pub static MAKING_UP_TIME: Color = Color::RGB(255, 90, 90); // color used when behind but gaining
pub static LOSING_TIME: Color = Color::RGB(135, 255, 135); // color used when ahead but losing

// state of timer, might implement real state switching eventually
#[derive(Debug)]
pub enum TimerState {
    OffsetCountdown { amt: u128 },
    Running { timestamp: u128 },
    Paused { time: u128, split: u128, time_str: String },
    NotStarted { time_str: String },
    Finished { time_str: String },
}

pub fn save_check() -> bool {
	match tfd::message_box_yes_no("Save run?", "Your run was a PB; do you want to save it?", tfd::MessageBoxIcon::Question, tfd::YesNo::Yes) {
		tfd::YesNo::No => {return false;}
		tfd::YesNo::Yes => {return true;}
	}
}

// gross way to make sure a split file is chosein for now until
// i set up a persistent file to remember what was last opened
pub fn open_splits() -> String {
    	let cwd = env::current_dir().unwrap();
    	let dir = cwd.to_str().unwrap();
    	println!("{:?}", cwd);
    	let mut path: Option<String> = None;
    	while path == None {
		path = tfd::open_file_dialog("Open split file", &dir, Some((&[".msf"], "")));
    	}
    	return path.unwrap();
}
