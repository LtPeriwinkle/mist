// starts application by instantiating App and running it

// comment the below line to get printing on windows
#![windows_subsystem = "windows"]
extern crate sdl2;

mod app;
mod comparison;
mod state;
mod config;
mod render;
mod splits;
use app::App;
use mist_core::dialogs::error;

fn main() {
    let context = sdl2::init().unwrap_or_else(|err| {
        error(&err);
    });
    let mut app = App::init(context).unwrap_or_else(|err| {error(&err);});
    app.run().unwrap_or_else(|err| {error(&err);});
}
