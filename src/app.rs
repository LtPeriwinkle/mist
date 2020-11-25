use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{Texture, WindowCanvas};
use sdl2::surface::Surface;
use sdl2::ttf;
use std::thread;
use std::time::{Duration, Instant};

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

// state of timer, a finished and notstarted will likely be added
enum TimerState {
    Running { color: Color, timestamp: u32 },
    Paused { time: u128, time_str: String },
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
            state: TimerState::Paused {
                time: 0,
                time_str: "0.000".to_string(),
            }, // might be a notstarted variant sometime down the line
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
        self.canvas.window_mut().set_minimum_size(0, timer_height + 10);

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

        for item in splits[0..bottom_split_index].iter() {
            on_screen.push(item);
        }

        for item in split_times[0..bottom_split_index].iter() {
            on_screen_times.push(item);
        }

        // set up variables used in the mainloop
        let mut frame_time: Instant;
        let mut total_time = Instant::now();
        let mut time_str: String;
        let mut before_pause: Option<Duration> = None;
        let one_sixtieth = Duration::new(0, 1_000_000_000 / 60);
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
                            on_screen = vec![];
                            on_screen_times = vec![];
                            let mut index = bottom_split_index - max_splits;
                            while index < bottom_split_index {
                                on_screen.push(&splits[index]);
                                on_screen_times.push(&split_times[index]);
                                index += 1;
                            }
                        }
                    }
                    // if scroll up and there are enough splits in the list, scroll splits up
                    Event::MouseWheel { y: 1, .. } => {
                        if bottom_split_index != max_splits {
                            bottom_split_index -= 1;
                            on_screen = vec![];
                            on_screen_times = vec![];
                            let mut index = bottom_split_index - max_splits;
                            while index < bottom_split_index {
                                on_screen.push(&splits[index]);
                                on_screen_times.push(&split_times[index]);
                                index += 1;
                            }
                        }
                    }
                    // enter as placeholder for stop/start, will be configurable eventually
                    Event::KeyDown {
                        keycode: Some(Keycode::Return),
                        timestamp: event_time,
                        repeat: false,
                        ..
                    } => {
                        // if the timer is paused, tell it to run and set the timestamp of when it was started
                        if let TimerState::Paused { time, .. } = self.state {
                            total_time = Instant::now();
                            before_pause = Some(Duration::from_millis(time as u64));
                            self.state = TimerState::Running {
                                color: Color::GREEN,
                                timestamp: event_time,
                            };
                        // if the timer is already running, pause it and calculate the display time
                        } else if let TimerState::Running {
                            timestamp: start_running_time,
                            ..
                        } = self.state
                        {
                            // if the timer was running before it was paused, add the time it ran for to the displayed time
                            match before_pause {
                                Some(x) => {
                                    self.state = TimerState::Paused {
                                        time: (event_time - start_running_time) as u128
                                            + x.as_millis(),
                                        time_str: timing::ms_to_readable(
                                            (event_time - start_running_time) as u128
                                                + x.as_millis(),
                                            true,
                                        ),
                                    };
                                }
                                None => {
                                    self.state = TimerState::Paused {
                                        time: (event_time - start_running_time) as u128,
                                        time_str: timing::ms_to_readable(
                                            (event_time - start_running_time) as u128,
                                            true,
                                        ),
                                    };
                                }
                            }
                        }
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::R),
                        ..
                    } => {
                        self.state = TimerState::Paused {
                            time: 0,
                            time_str: "0.000".to_string(),
                        };
                    },
                    Event::Window { win_event: WindowEvent::Resized(_, y), .. } => {
			let height = self.canvas.viewport().height();
			let rows_height = (max_splits as u32 * (splits_height + 5));
			if height - timer_height < rows_height {
				let diff = ((rows_height - (height - timer_height)) / splits_height);// + 1;
				println!("{}", diff);
				if max_splits > diff as usize {
					max_splits -= diff as usize;
                       			if bottom_split_index != max_splits {
                           			 bottom_split_index -= 1;
                           			 on_screen = vec![];
                           			 on_screen_times = vec![];
                            		 	 let mut index = bottom_split_index - max_splits;
                            			 while index < bottom_split_index {
                                			on_screen.push(&splits[index]);
                                			on_screen_times.push(&split_times[index]);
                                			index += 1;
                            			}
                        		}
				}
			}
		    },
                    _ => {}
                }
            }
            let window_width = self.canvas.viewport().width();
            render::render_rows(&on_screen, &on_screen_times, &mut self.canvas, window_width);
            let color: Color;
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
            if Instant::now().duration_since(frame_time) <= one_sixtieth {
                thread::sleep(
                    // if the entire loop pass was completed in under 1/60 second, delay to keep the framerate at ~60fps
                    Duration::new(0, 1_000_000_000 / 60)
                        - Instant::now().duration_since(frame_time),
                );
            }
        }
    }
    // updates time string based on timer state, basically leaves it the same if timer is paused
    fn update_time(&self, before_pause: Option<Duration>, total_time: Instant) -> String {
        let time: String;
        match &self.state {
            TimerState::Running { .. } => { 
          	match before_pause {
                    Some(x) => {
                        time = timing::ms_to_readable(
                            total_time.elapsed().as_millis() + x.as_millis(),
                            false,
                        );
                    }
                    None => {
                        time = timing::ms_to_readable(total_time.elapsed().as_millis(), false);
                    }
          	}
            }
            TimerState::Paused { time_str: display, .. } => {
            time = display.to_string();
            }
        }
        return time;
    }
}
