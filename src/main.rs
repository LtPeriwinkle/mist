#![windows_subsystem = "windows"]
extern crate sdl2;

mod app;
mod render;
mod splits;
mod timing;
use app::App;

fn main() {
    let context = sdl2::init().expect("could not initialize SDL");
    let mut app = App::init(context);
    app.run();
}
