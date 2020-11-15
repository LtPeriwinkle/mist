extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::{Event};
use sdl2::ttf;
use sdl2::keyboard::Keycode;
use std::time::{Instant, Duration};
use std::thread;

mod timing;
mod render;
 
fn main() {
	let sdl_context = sdl2::init().unwrap();
	let ttf_context = ttf::init().unwrap();
	let video_subsystem = sdl_context.video().unwrap();
 
	let window = video_subsystem.window("mist", 300, 500)
		.position_centered()
		.resizable()
		.build()
		.unwrap();
 
	let mut canvas = window.into_canvas().build().unwrap();
	let mut window_width = canvas.viewport().width();
 
	let creator = canvas.texture_creator();
	let font = ttf_context.load_font("segoe-ui-bold.ttf", 40).unwrap();

	canvas.clear();
	let test = ["Something", "else", "words", "text", "split 5 idk", "q", "asdf", "words 2", "no", "yes", "another one"];
	let mut on_screen = vec![];
	let mut current_index = 8;
	let original_index = current_index;
	for item in test[0..current_index].iter() {
		let text_surface = font.render(item).blended(Color::WHITE).unwrap();
		let texture = creator.create_texture_from_surface(&text_surface).unwrap();
		on_screen.push(texture);
	}
	render::render_rows(&on_screen, &mut canvas, window_width);
	let mut event_pump = sdl_context.event_pump().unwrap();
	canvas.present();
	let mut frame_time: Instant;
	thread::spawn(|| {
		timing::time_30_fps();
	});
	'running: loop {
		frame_time = Instant::now();
		canvas.clear();
		for event in event_pump.poll_iter() {
			if let Event::KeyDown { scancode, .. } = event {
				println!("{:?}", scancode);
			}
			match event {
				Event::Quit {..} |
				Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
					break 'running
				},
				Event::KeyDown { keycode: Some(Keycode::Space), .. } | Event::MouseWheel { y: -1, .. } => {
					if current_index < test.len() {
						current_index += 1;
						on_screen = vec![];
						for item in test[current_index - original_index..current_index].iter() {
							let text_surface = font.render(item).blended(Color::WHITE).unwrap();
							let texture = creator.create_texture_from_surface(&text_surface).unwrap();
							on_screen.push(texture);
						}
					}
					//println!("{}", current_index);
				},
				Event::MouseWheel { y: 1, .. } => {
					if current_index != original_index {
						current_index -= 1;
						on_screen = vec![];
						for item in test[current_index - original_index..current_index].iter() {
							let text_surface = font.render(item).blended(Color::WHITE).unwrap();
							let texture = creator.create_texture_from_surface(&text_surface).unwrap();
							on_screen.push(texture);
						}
					}
					//println!("{}", current_index);
				}
				_ => {}
			}
		}
		window_width = canvas.viewport().width();
		render::render_rows(&on_screen, &mut canvas, window_width);
		canvas.present();
		thread::sleep(Duration::new(0, 1_000_000_000 / 60) - Instant::now().duration_since(frame_time));
	}
}