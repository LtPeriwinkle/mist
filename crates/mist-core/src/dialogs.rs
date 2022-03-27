//! Dialog boxes to prompt the user for things.
//!
//! Uses [tinyfiledialogs] to provide dialog boxes, which is cross-platform and can even work
//! in a terminal if none of the dialog APIs it's expecting are available.
#[cfg(feature = "config")]
use crate::config::Config;
use std::fs::File;
use tinyfiledialogs::{
    message_box_ok, message_box_yes_no, open_file_dialog, save_file_dialog_with_filter,
    MessageBoxIcon, YesNo,
};

/// Check if the user wants to save their modified split file.
///
/// If they click yes, return `true`. No returns `false`.
pub fn save_check() -> bool {
    match message_box_yes_no(
        "Save run?",
        "Your split file has been updated, do you want to save it?",
        MessageBoxIcon::Question,
        YesNo::Yes,
    ) {
        YesNo::Yes => true,
        YesNo::No => false,
    }
}

/// Open a file select dialog box.
///
/// Box title will be `title`. `filter` should be formatted like `*.msf` to filter for msf file extensions etc.
/// Returns `None` if the user closes the dialog box or presses Cancel.
pub fn get_file(title: &str, filter: &str) -> Option<String> {
    open_file_dialog(title, "", Some((&[filter], "")))
}

/// Open a save as dialog box.
///
/// Returns `None` if the user closes the dialog box or presses Cancel.
pub fn get_save_as() -> Option<String> {
    save_file_dialog_with_filter("Save as:", "", &["*.msf"], "mist split files")
}

/// Ask the user if they want to try another file.
pub fn try_again() -> bool {
    match message_box_yes_no(
        "File parse failed",
        "File parse failed. Do you want to try another?",
        MessageBoxIcon::Question,
        YesNo::Yes,
    ) {
        YesNo::Yes => true,
        YesNo::No => false,
    }
}

/// Get the path of an msf file to use.
///
/// Returns [`None`] if the user cancels the dialog box
pub fn get_run_path() -> Option<String> {
    get_file("Open split file", "*.msf")
}

/// Gets the path of a [`Config`] and attempts to parse it.
///
/// # Errors
///
/// * If the file cannot be read or there is another fs error.
///
/// # Nones
///
/// * If the user does not select a file.
/// * If the file selected cannot be parsed into a [`Config`].
#[cfg(feature = "config")]
pub fn open_config() -> Result<Option<Config>, String> {
    loop {
        match get_file("Open a config file", "*.cfg") {
            Some(ref p) => {
                let f = File::open(p).map_err(|e| e.to_string())?;
                let config: Result<Config, String> =
                    ron::de::from_reader(f).map_err(|e| e.to_string());
                match config {
                    Ok(c) => {
                        return Ok(Some(c));
                    }
                    Err(_) => {
                        if !try_again() {
                            return Ok(None);
                        }
                    }
                }
            }
            None => return Ok(None),
        }
    }
}

/// Inform the user of an error, then exit the program.
///
/// Only used at the top level of the call stack in mist. Do not go using this in places.
pub fn error(err: &str) -> ! {
    let err = err.replace('\'', "").replace('"', "");
    message_box_ok("Error", &err, MessageBoxIcon::Error);
    std::process::exit(1)
}
