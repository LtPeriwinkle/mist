extern crate sdl2;

mod timing;
mod render;
mod app;
use app::App;

fn main() {
	let mut app = App::init();
	app.run();
}