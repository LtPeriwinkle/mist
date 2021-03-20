// miscellaneous stuff that doesnt really fit anywhere else
use mist_run_utils::run::Run;
use std::env;
use tinyfiledialogs as tfd;

//state of timer, might implement real state switching eventually
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
        "Your splits have been updated; do you want to save them?",
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

// get a new split file path from the user and try to load a run from it, then return
// both the run and the path
pub fn reload_splits() -> Option<(Run, String)> {
    let mut path: Option<String>;
    loop {
        path = open_file("Open split file", "*.msf");
        match path {
            None => return None,
            Some(ref p) => match Run::from_msf_file(&p) {
                Some(r) => return Some((r, path.unwrap())),
                None => {
                    if !bad_file_dialog("Split file parse failed. Try another file?") {
                        return None;
                    }
                }
            },
        }
    }
}
