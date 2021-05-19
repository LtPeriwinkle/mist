use tinyfiledialogs::{YesNo, message_box_yes_no, message_box_ok, MessageBoxIcon, open_file_dialog};
use std::fs::File;
use std::io::{Error, BufReader};
use crate::run::Run;
use crate::parse::MsfParser;

pub fn save_check() -> bool {
    match message_box_yes_no("Save run?", "Your split file has been updated, do you want to save it?", MessageBoxIcon::Question, YesNo::Yes) {
        YesNo::Yes => true,
        YesNo::No => false
    }
}

fn get_file(title: &str, filter: &str) -> Option<String> {
    open_file_dialog(title, "", Some((&[filter], "")))
}

fn try_again() -> bool {
    match message_box_yes_no("Split file parse failed", "Split file parse failed. Do you want to try another?", MessageBoxIcon::Question, YesNo::Yes) {
        YesNo::Yes => true,
        YesNo::No => false
    }
}

pub fn open_run() -> Result<Option<(Run, String)>, Error> {
    loop {
        match get_file("Open split file", "*.msf") {
            Some(ref p) => {
                let f = File::open(p)?;
                let reader = BufReader::new(f);
                let parser = MsfParser::new();
                match parser.parse(reader) {
                    Ok(r) => {
                        return Ok(Some((r, p.to_owned())));
                    }
                    Err(_) => {
                        if !try_again() {
                            return Ok(None);
                        }
                    }
                }
            }
            None => return Ok(None)
        }
    }
}

pub fn error(err: &str) -> ! {
    message_box_ok("Error", err, MessageBoxIcon::Error);
    std::process::exit(1)
}

