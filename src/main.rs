// starts application by instantiating App and running it

// comment the below line to get printing on windows
#![windows_subsystem = "windows"]

mod app;
mod comparison;
mod keybinds;
mod panels;
mod render;
mod splits;
mod state;
use app::App;
use mist_core::dialogs::error;

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let mut out = String::from("mist panicked: ");
        if let Some(payload) = info.payload().downcast_ref::<String>() {
            out.push_str(&format!("({})", payload));
        }
        if let Some(loc) = info.location() {
            out.push_str(&format!(" in {}:{}", loc.file(), loc.line()));
        }
        error(&out);
    }));
    let context = sdl2::init().unwrap_or_else(|err| {
        error(&err);
    });
    let mut app = App::init(context).unwrap_or_else(|err| {
        error(&err);
    });
    app.run().unwrap_or_else(|err| {
        error(&err);
    });
}
