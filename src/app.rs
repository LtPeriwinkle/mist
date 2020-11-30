use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{Texture, WindowCanvas};
use sdl2::surface::Surface;
use sdl2::ttf;
use std::thread;
use std::time::{Duration, Instant};
use std::mem;

use crate::render;
use crate::splits::{self, Run};
use crate::timing;

const SPLITS_ON_SCREEN: usize = 8; //used to limit number of splits displayed

// struct that holds information about the running app and its state
#[allow(dead_code)]
pub struct App {
    context: sdl2::Sdl,
    ev_pump: sdl2::EventPump,
    canvas: WindowCanvas,
    ttf: sdl2::ttf::Sdl2TtfContext,
    state: TimerState,
    run: splits::Run,
}

// state of timer
#[derive(Debug)]
enum TimerState {
    Running { timestamp: u32 },
    Paused { time: u128, time_str: String},
    NotStarted,
    Finished {time_str: String}
}

impl App {
    pub fn init(context: sdl2::Sdl) -> Self {
        // sdl setup boilerplate
        let video = context.video().expect("could not initialize SDL video");
        let window = video
            .window("mist", 300, 500)
            .position_centered()
            .resizable()
            .build()
            .expect("could not initialize SDL window");
        let canvas = window
            .into_canvas()
            .build()
            .expect("could not initialize SDL canvas");
        let ttf = ttf::init().expect("could not initialize TTF subsystem");
        let ev_pump = context
            .event_pump()
            .expect("could not initialize SDL event handler");
        App {
            context,
            ev_pump,
            canvas,
            ttf,
            state: TimerState::NotStarted,
            run: Run::new(),
        }
    }

    pub fn run(&mut self) {
        // set up some stuff that's a pain to do elsewhere
        self.canvas.clear();
        let mut bottom_split_index = SPLITS_ON_SCREEN;
        let mut max_splits: usize;
        let timer_font = self
            .ttf
            .load_font("assets/segoe-ui-bold.ttf", 60)
            .expect("could not open font file");
        let font = self
            .ttf
            .load_font("assets/segoe-ui-bold.ttf", 25)
            .expect("could not open font file");
        let creator = self.canvas.texture_creator();
        let timer_height = timer_font.size_of("0123456789").unwrap().1;
        let splits_height = font.size_of("qwertyuiopasdfghjklzxcvbnm").unwrap().1;
        self.canvas
            .window_mut()
            .set_minimum_size(0, timer_height + 10)
            .unwrap();

        // get first vec of split name textures
        self.run = Run::from_file("test.msf");
        let split_names = &self.run.splits;
        let split_times_ms: Vec<u128> = self.run.best_times.iter().cloned().collect();
        let split_times_raw: Vec<String> = timing::split_time_sum(split_times_ms);
        let mut text_surface: Surface;
        let mut texture: Texture;
        let mut on_screen: Vec<&Texture> = vec![];
        let mut on_screen_times: Vec<&Texture> = vec![];
        let mut splits: Vec<Texture> = vec![];
        let mut split_times: Vec<Texture> = vec![];

        // set up max splits dynamically in case there are too few splits
        if SPLITS_ON_SCREEN > split_names.len() {
            bottom_split_index = split_names.len();
            max_splits = split_names.len();
        } else {
            max_splits = SPLITS_ON_SCREEN;
        }
        for item in split_names {
            text_surface = font
                .render(item)
                .blended(Color::WHITE)
                .expect("split name font render failed");
            texture = creator
                .create_texture_from_surface(text_surface)
                .expect("split name texture creation failed");
            splits.push(texture);
        }

        for item in split_times_raw {
            text_surface = font
                .render(&item)
                .blended(Color::WHITE)
                .expect("split time font render failed");
            texture = creator
                .create_texture_from_surface(text_surface)
                .expect("split time texture creation failed");
            split_times.push(texture);
        }

        // set up variables used in the mainloop
        // framerate cap timer
        let mut frame_time: Instant;
        let mut total_time = Instant::now();
        // display time
        let mut time_str: String;
        // keeps track of whether timer has been paused and paused value
        let mut before_pause: Option<u128> = None;
        // this one should be a static but duration isnt allowed to be static apparently
        let one_sixtieth = Duration::new(0, 1_000_000_000 / 60);
        let mut current_split = 0;
        // these two to avoid having to drop and reallocate every loop
        let mut window_width: u32;
        let mut color: Color;
        // sum of split times for display on rows
        let mut recreate_on_screen: Option<bool> = Some(true);
        let mut diff: u32 = 0;
        let mut len: usize = splits.len();
        self.canvas.present();

        // main loop
        'running: loop {
            frame_time = Instant::now();
            self.canvas.set_draw_color(Color::BLACK);
            self.canvas.clear();
            for event in self.ev_pump.poll_iter() {
                // print events to terminal if running in debug
                #[cfg(debug_assertions)]
                println!("{:?}", event);

                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    // if scroll down and there are enough splits, scroll splits down
                    Event::MouseWheel { y: -1, .. } => {
                        if bottom_split_index < splits.len() {
                            bottom_split_index += 1;
                            let index = bottom_split_index - max_splits;
                            on_screen = splits[index..bottom_split_index].iter().collect();
                            on_screen_times =
                                split_times[index..bottom_split_index].iter().collect();
                        }
                    }
                    // if scroll up and there are enough splits in the list, scroll splits up
                    Event::MouseWheel { y: 1, .. } => {
                        if bottom_split_index != max_splits {
                            bottom_split_index -= 1;
                            let index = bottom_split_index - max_splits;
                            on_screen = splits[index..bottom_split_index].iter().collect();
                            on_screen_times =
                                split_times[index..bottom_split_index].iter().collect();
                        }
                    }
                    // enter as placeholder for pause/continue
                    Event::KeyDown {
                        keycode: Some(Keycode::Return),
                        timestamp: event_time,
                        repeat: false,
                        ..
                    } => {
			match self.state {
				TimerState::Paused {time: t, ..} => {
    					total_time = Instant::now();
					before_pause = Some(t);
					self.state = TimerState::Running {timestamp: event_time};
				},
				TimerState::Running {timestamp: t} => {
					self.state = TimerState::Paused { time: (event_time - t) as u128 + before_pause.unwrap_or(0), time_str: timing::ms_to_readable((event_time - t) as u128 + before_pause.unwrap_or(0), true) };
				},
				_ => {}
			}
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::R),
                        ..
                    } => {
                        self.state = TimerState::NotStarted;
                    }
                    Event::Window {
                        win_event: WindowEvent::Resized(..),
                        ..
                    } => {
                        let height = self.canvas.viewport().height();
                        let rows_height = max_splits as u32 * (splits_height + 5);
                        len = splits.len();
                        if height - timer_height < rows_height {
                            diff = (rows_height - (height - timer_height)) / splits_height;
                            recreate_on_screen = Some(false);
                        } else if rows_height < height - timer_height {
                            diff = ((height - timer_height) - rows_height) / splits_height;
                            if !(max_splits + diff as usize > SPLITS_ON_SCREEN
                                || max_splits + diff as usize > splits.len())
                            {
                                recreate_on_screen = Some(false);
                            }
                        }
                    }
                    Event::KeyDown {keycode: Some(Keycode::Space), timestamp: event_time, ..} => {
			match self.state {
				TimerState::NotStarted => {
					total_time = Instant::now();
					self.state = TimerState::Running {timestamp: event_time};
					current_split = 0;
				},
				TimerState::Running {timestamp: t, ..} => {
    					time_str = timing::ms_to_readable((event_time - t) as u128 + before_pause.unwrap_or(0), true);
    					text_surface = font.render(&time_str).blended(Color::WHITE).unwrap();
    					texture = creator.create_texture_from_surface(&text_surface).unwrap();
    					on_screen_times = vec![];
    					mem::replace(&mut split_times[current_split], texture);
					if current_split < splits.len() {
						current_split += 1;
					} else {
						self.state = TimerState::Finished {time_str};
					}
				},
				_ => {}
			}
                    }
                    _ => {}
                }
            }
            window_width = self.canvas.viewport().width();
            match recreate_on_screen {
		Some(true) => {
			on_screen = splits[0..bottom_split_index].iter().collect();
			on_screen_times = split_times[0..bottom_split_index].iter().collect();
			recreate_on_screen = None;
		},
		Some(false) => {
                        if max_splits > diff as usize {
                            max_splits -= diff as usize;
                            if current_split + max_splits > len {
                                bottom_split_index = len;
                                on_screen = splits[len - max_splits..bottom_split_index]
                                    .iter()
                                    .collect();
                                on_screen_times = split_times
                                    [len - max_splits..bottom_split_index]
                                    .iter()
                                    .collect();
                            } else if current_split < max_splits {
                                bottom_split_index = max_splits;
                                on_screen = splits[0..max_splits].iter().collect();
                                on_screen_times = split_times[0..max_splits].iter().collect();
                            } else if current_split >= max_splits {
                                bottom_split_index = current_split + max_splits;
                                on_screen = splits[current_split..current_split + max_splits]
                                    .iter()
                                    .collect();
                                on_screen_times = split_times
                                    [current_split..current_split + max_splits]
                                    .iter()
                                    .collect();
                            }
                        }
			recreate_on_screen = None;
		},
		_ => {}
            }
            render::render_rows(&on_screen, &on_screen_times, &mut self.canvas, window_width);
            if let TimerState::Running { .. } = self.state {
                // will eventually calculate whether run is ahead/behind/gaining/losing and adjust appropriately
                color = Color::GREEN;
            } else {
                color = Color::WHITE;
            }
            time_str = self.update_time(before_pause, total_time);
            text_surface = timer_font
                .render(&time_str)
                .shaded(color, Color::BLACK)
                .expect("time font render failed");
            texture = creator
                .create_texture_from_surface(&text_surface)
                .expect("time texture creation failed");
            render::render_time(&texture, &mut self.canvas);
            self.canvas.present();
            //println!("{:?}", self.state);
            if Instant::now().duration_since(frame_time) <= one_sixtieth {
                thread::sleep(
                    // if the entire loop pass was completed in under 1/60 second, delay to keep the framerate at ~60fps
                    one_sixtieth
                        - Instant::now().duration_since(frame_time),
                );
            }
        }
    }
    // updates time string based on timer state, basically leaves it the same if timer is paused
    fn update_time(&self, before_pause: Option<u128>, total_time: Instant) -> String {
        let time: String;
        match &self.state {
            TimerState::Running { .. } => {
		time = timing::ms_to_readable(total_time.elapsed().as_millis() + before_pause.unwrap_or(0), false);
            },
            TimerState::Paused {
                time_str: display, ..
            } => {
                time = display.to_string();
            },
            TimerState::NotStarted {} => {
		time = "0.000".to_string();
            },
            TimerState::Finished {time_str: string} => {
		time = string.to_owned();
            }
        }
        return time;
    }
}
