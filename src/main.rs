// starts application by instantiating App and running it

// comment the below line to get printing on windows
#![windows_subsystem = "windows"]
extern crate sdl2;

mod app;
mod comparison;
mod components;
mod config;
mod render;
mod splits;
mod timing;
use app::App;
use components::error_dialog;

fn main() {
    let context = sdl2::init().unwrap_or_else(|err| {
        error_dialog(err.to_string());
        unreachable!();
    });
    let mut app = App::init(context);
    app.run();
}
