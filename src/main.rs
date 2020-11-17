extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::{Event};
use sdl2::ttf;
use sdl2::keyboard::Keycode;
use std::time::{Instant, Duration};
use std::thread;

mod timing;
mod render;
use timing::TimeUpdateEvent;

const SPLITS_ON_SCREEN: usize = 8; //used to limit number of splits displayed

fn main() {
	//sdl boilerplate
	let sdl_context = sdl2::init().expect("sdl init failed");
	//event setup
	let event_subsystem = sdl_context.event().unwrap();
	event_subsystem.register_custom_event::<TimeUpdateEvent>().unwrap();
	let ev_sender = event_subsystem.event_sender();
	//font setup
	let ttf_context = ttf::init().expect("ttf init failed");
	let font = ttf_context.load_font("assets/segoe-ui-bold.ttf", 30).unwrap();
	let timer_font = ttf_context.load_font("assets/segoe-ui-bold.ttf", 60).unwrap();
	//window and canvas for drawing
	let video_subsystem = sdl_context.video().unwrap();
	let window = video_subsystem.window("mist", 300, 500)
		.position_centered()
		.resizable()
		.build()
		.unwrap();
	let mut canvas = window.into_canvas().build().unwrap();
	let mut window_width = canvas.viewport().width();
 	let creator = canvas.texture_creator();

	canvas.clear();

	//queue and render the initial splits on screen
	//these splits will eventually come from a splits file
	let splits = ["Something", "else", "words", "text", "split 5 idk", "q", "asdf", "words 2", "no", "yes", "another one"];
	let mut on_screen = vec![];
	let mut current_index = SPLITS_ON_SCREEN;
	for item in splits[0..current_index].iter() {
		let text_surface = font.render(item).blended(Color::WHITE).unwrap();
		let texture = creator.create_texture_from_surface(&text_surface).unwrap();
		on_screen.push(texture);
	}
	render::render_rows(&on_screen, &mut canvas, window_width);

	thread::spawn(|| {
		timing::time_30_fps(ev_sender); //separate thread for timing so we dont have to hope mainloop is fast enough
	});

	let mut event_pump = sdl_context.event_pump().unwrap();
	let mut frame_time: Instant;
	let mut time_ev = "0.000".to_string();
	let mut cleared = false;
	render::render_time(&time_ev, &mut canvas, &timer_font, &creator);
	canvas.present();
	'running: loop {
		frame_time = Instant::now();
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
					if current_index < splits.len() {
						current_index += 1;
						cleared = true;
						canvas.clear();
						on_screen = vec![];
						for item in splits[current_index - SPLITS_ON_SCREEN..current_index].iter() {
							let text_surface = font.render(item).blended(Color::WHITE).unwrap();
							let texture = creator.create_texture_from_surface(&text_surface).unwrap();
							on_screen.push(texture);
						}
					}
					//println!("{}", current_index);
				},
				Event::MouseWheel { y: 1, .. } => {
					if current_index != SPLITS_ON_SCREEN {
						current_index -= 1;
						cleared = true;
						canvas.clear();
						on_screen = vec![];
						for item in splits[current_index - SPLITS_ON_SCREEN..current_index].iter() {
							let text_surface = font.render(item).blended(Color::WHITE).unwrap();
							let texture = creator.create_texture_from_surface(&text_surface).unwrap();
							on_screen.push(texture);
						}
					}
					//println!("{}", current_index);
				}
				Event::User {..} => {
					time_ev = event.as_user_event_type::<TimeUpdateEvent>().unwrap().time;
					cleared = true;
					canvas.clear();
				}
				_ => {}
			}
		}
		if cleared {
			window_width = canvas.viewport().width();
			render::render_rows(&on_screen, &mut canvas, window_width);
			render::render_time(&time_ev, &mut canvas, &timer_font, &creator);
			cleared = false;
		}
		canvas.present();
		thread::sleep(Duration::new(0, 1_000_000_000 / 60) - Instant::now().duration_since(frame_time));
	}
}