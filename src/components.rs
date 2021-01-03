// miscellaneous stuff that doesnt really fit anywhere else
use sdl2::pixels::Color;
use std::env;
use tinyfiledialogs as tfd;

pub const SPLITS_ON_SCREEN: usize = 8; // max splits allowed on screen

pub static AHEAD: Color = Color::RGB(0, 255, 0);
pub static BEHIND: Color = Color::RGB(255, 0, 0);
pub static MAKING_UP_TIME: Color = Color::RGB(255, 90, 90); // color used when behind but gaining
pub static LOSING_TIME: Color = Color::RGB(135, 255, 135); // color used when ahead but losing
pub static GOLD: Color = Color::RGB(255, 255, 0); // default for when beating best split time
// state of timer, might implement real state switching eventually
#[derive(Debug)]
pub enum TimerState {
    OffsetCountdown {
        amt: u128,
    },
    Running {
        timestamp: u128,
    },
    Paused {
        time: u128,
        split: u128,
        time_str: String,
    },
    NotStarted {
        time_str: String,
    },
    Finished {
        time_str: String,
    },
}

// open a dialog box to ask the user if they want to save the run they just completed to its original split file
pub fn save_check() -> bool {
    match tfd::message_box_yes_no(
        "Save run?",
        "Your run was a PB; do you want to save it?",
        tfd::MessageBoxIcon::Question,
        tfd::YesNo::Yes,
    ) {
        tfd::YesNo::No => {
            return false;
        }
        tfd::YesNo::Yes => {
            return true;
        }
    }
}

// open a generic file dialog box with the passed filter and title in order to get a file path
pub fn open_file(title: &str, ext: &str) -> Option<String> {
    let cwd = env::current_dir().unwrap();
    let mut dir = cwd.to_string_lossy();
    dir.to_mut().push_str("/");
    let path = tfd::open_file_dialog(title, &dir, Some((&[ext], "")));
    return path;
}

// open a dialog box to alert the user that their file was invalid and ask them what they would like to do
pub fn bad_file_dialog(err: &str) -> bool {
    match tfd::message_box_ok_cancel(
        "file read error",
        err,
        tfd::MessageBoxIcon::Error,
        tfd::OkCancel::Ok,
    ) {
        tfd::OkCancel::Ok => true,
        tfd::OkCancel::Cancel => false,
    }
}

// wrapper function so the tfd stuff can all stay in this file
pub fn info_dialog(title: &str, text: &str) {
    tfd::message_box_ok(title, text, tfd::MessageBoxIcon::Info);
}
