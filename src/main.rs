// comment the below line to get printing on windows
#![windows_subsystem = "windows"]

mod app;
mod keybinds;
mod panels;
mod render;
mod splits;
use app::App;
use mist_core::{config::Config, dialogs::error};
use sdl2::rwops::RWops;

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let out = info.to_string();
        println!("{}", out);
        error(&out);
    }));
    let context = sdl2::init().unwrap_or_else(|err| {
        error(&err);
    });
    let ttf = sdl2::ttf::init().unwrap();
    let config = Config::open().unwrap();
    let tfont = config.tfont();
    let tf_bytes = tfont.get_bytes().unwrap();
    let sfont = config.sfont();
    let sf_bytes = sfont.get_bytes().unwrap();
    let rw = RWops::from_bytes(&tf_bytes.0).unwrap();
    let timer_font = ttf
        .load_font_at_index_from_rwops(rw, tf_bytes.1, tfont.size())
        .unwrap();
    let rw = RWops::from_bytes(&sf_bytes.0).unwrap();
    let splits_font = ttf
        .load_font_at_index_from_rwops(rw, sf_bytes.1, sfont.size())
        .unwrap();
    let app = App::init(context, &timer_font, &splits_font).unwrap_or_else(|err| {
        error(&err);
    });
    app.run(&timer_font, &splits_font).unwrap_or_else(|err| {
        error(&err);
    });
}
