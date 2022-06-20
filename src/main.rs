// comment the below line to get printing on windows
#![windows_subsystem = "windows"]

mod app;
mod keybinds;
mod panels;
mod render;
mod splits;
use app::App;
use mist_core::dialogs::error;

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let out = info.to_string();
        println!("{}", out);
        error(&out);
    }));
    let context = sdl2::init().unwrap_or_else(|err| {
        error(&err);
    });
    let app = App::init(context).unwrap_or_else(|err| {
        error(&err);
    });
    app.run().unwrap_or_else(|err| {
        error(&err);
    });
}
