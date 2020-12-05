use sdl2::event::{Event, WindowEvent};
use sdl2::image::LoadSurface;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{Texture, WindowCanvas};
use sdl2::surface::Surface;
use sdl2::ttf;
use std::thread;
use std::time::{Duration, Instant};

use crate::components::*;
use crate::render;
use crate::splits::{self, Run};
use crate::timing;

static mut RECREATE_DEFAULT: Option<u8> = Some(3); // used to determine whether to recreate slice every loop

// struct that holds information about the running app and its state
#[allow(dead_code)]
pub struct App {
    context: sdl2::Sdl,
    ev_pump: sdl2::EventPump,
    timer: sdl2::TimerSubsystem,
    canvas: WindowCanvas,
    ttf: sdl2::ttf::Sdl2TtfContext,
    state: TimerState,
    run: splits::Run,
}

impl App {
    pub fn init(context: sdl2::Sdl) -> Self {
        // sdl setup boilerplate
        let video = context.video().expect("could not initialize SDL video");
        let mut window = video
            .window("mist", 300, 500)
            .position_centered()
            .resizable()
            .build()
            .expect("could not initialize SDL window");
        let icon = Surface::from_file("assets/MIST.png").expect("could not load icon");
        window.set_icon(icon);
        let canvas = window
            .into_canvas()
            .build()
            .expect("could not initialize SDL canvas");
        let ttf = ttf::init().expect("could not initialize TTF subsystem");
        let ev_pump = context
            .event_pump()
            .expect("could not initialize SDL event handler");
        let timer = context.timer().unwrap();
        App {
            context,
            ev_pump,
            timer,
            canvas,
            ttf,
            state: TimerState::NotStarted {
                time_str: "".to_string(),
            },
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
        let splits_height = font.size_of("qwertyuiopasdfghjklzxcvbnm01234567890").unwrap().1;
        // set the minimum height of the window to the size of the time texture
        self.canvas
            .window_mut()
            .set_minimum_size(0, timer_height + 10)
            .unwrap();

        // get first vec of split name textures from file
        self.run = Run::from_file("run.msf");
        let split_names = &self.run.splits;
        let offset = self.run.offset;
        // if there is an offset, display it properly
        match offset {
            Some(x) => {
                self.state = TimerState::NotStarted {
                    time_str: format!("-{}", timing::ms_to_readable(x, false)),
                };
            }
            _ => {}
        }
        // get ms split times then convert them to pretty, summed times
        let split_times_ms: Vec<u128> = self.run.best_times.iter().cloned().collect();
        let summed_times = timing::split_time_sum(&split_times_ms);
        let split_times_raw: Vec<String> = summed_times
            .iter()
            .map(|val| timing::ms_to_readable(*val, true))
            .collect();
        // initialize variables that are used in the loop for replacing timer texture
        let mut text_surface: Surface;
        let mut texture: Texture;
        let mut on_screen: &[Texture] = &[];
        // vectors that hold the textures for split names and their associated times
        let mut splits: Vec<Texture> = vec![];
        let mut split_times: Vec<Texture> = vec![];

        // set up max splits dynamically in case there are too few splits
        if SPLITS_ON_SCREEN > split_names.len() {
            bottom_split_index = split_names.len();
            max_splits = split_names.len();
        } else {
            max_splits = SPLITS_ON_SCREEN;
        }

        // convert the split names into textures and add them to the split name vec
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

        // same as above but for times
        for item in split_times_raw {
            // blended text render does antialias and removes background box, is slower though
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
        let mut recreate_on_screen: Option<u8> = Some(0);
        // if there are no splits, dont waste time remaking the on screen ones every loop
        if self.run.splits.len() == 0 {
            unsafe {
                RECREATE_DEFAULT = None; // RECREATE_DEFAULT is mutable static, requires unsafe
            }
        }
        // diff between max on screen and current, used when resizing window
        let mut diff: u32 = 0;
        let mut len: usize = splits.len();
        // index of top split on screen
        let mut index: usize;
        // current split in the slice of splits sent to render_time()
        let mut cur: usize;
        let mut split_time_ticks = 0;
        let mut active_run_times: Vec<u128> = vec![];
        self.canvas.present();

        // main loop
        'running: loop {
            // start measuring the time this loop pass took
            frame_time = Instant::now();
            // remove stuff from the backbuffer and fill the space with black
            self.canvas.set_draw_color(Color::BLACK);
            self.canvas.clear();

            // if the timer is doing an offset, make sure it should still be negative
            // if it shouldnt, convert to running state
            if let TimerState::OffsetCountdown { amt } = self.state {
                if amt <= total_time.elapsed().as_millis() {
                    self.state = TimerState::Running {
                        timestamp: self.timer.ticks(),
                    };
                    split_time_ticks = self.timer.ticks();
                    total_time = Instant::now();
                }
            }

            // repeat stuff in here for every event that occured between frames
            // in order to properly respond to them
            for event in self.ev_pump.poll_iter() {
                // print events to terminal if running in debug
                #[cfg(debug_assertions)]
                println!("{:?}", event);

                match event {
                    // quit program on esc or being told by wm to close
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,

                    // if scroll down and there are enough splits, scroll splits down
                    Event::MouseWheel { y: -1, .. } => {
                        if bottom_split_index < splits.len() {
                            bottom_split_index += 1;
                            recreate_on_screen = Some(2);
                        }
                    }

                    // if scroll up and there are enough splits in the list, scroll splits up
                    Event::MouseWheel { y: 1, .. } => {
                        if bottom_split_index != max_splits {
                            bottom_split_index -= 1;
                            recreate_on_screen = Some(2);
                        }
                    }

                    // enter as placeholder for pause/continue
                    Event::KeyDown {
                        keycode: Some(Keycode::Return),
                        timestamp: event_time,
                        repeat: false,
                        ..
                    } => match self.state {
                        // if timer is paused, unpause it, put the amount of time before the pause in a variable
                        // and set the state to running
                        TimerState::Paused { time: t, .. } => {
                            total_time = Instant::now();
                            before_pause = Some(t);
                            self.state = TimerState::Running {
                                timestamp: event_time,
                            };
                        }
                        // if the timer is already running, set it to paused.
                        TimerState::Running { timestamp: t } => {
                            self.state = TimerState::Paused {
                                time: (event_time - t) as u128 + before_pause.unwrap_or(0),
                                time_str: timing::ms_to_readable(
                                    (event_time - t) as u128 + before_pause.unwrap_or(0),
                                    true,
                                ),
                            };
                        }
                        _ => {}
                    },

                    // R key to reset timer
                    Event::KeyDown {
                        keycode: Some(Keycode::R),
                        ..
                    } => {
                        active_run_times = vec![];
                        match offset {
                        	// if there is an offset, reset the timer to that, if not, reset timer to 0
                        	Some(x) => {
	                            before_pause = None;
        	                    self.state = TimerState::NotStarted {
                	                time_str: format!("-{}", timing::ms_to_readable(x, false)),
                        	    };
	                        }
        	                None => {
                	            self.state = TimerState::NotStarted {
                        	        time_str: "0.000".to_owned(),
      	                            };
              	               }
                        }
                    },

                    // handle vertical window resize by changing number of splits
                    Event::Window {
                        win_event: WindowEvent::Resized(..),
                        ..
                    } => {
                        let height = self.canvas.viewport().height();
                        let rows_height = max_splits as u32 * (splits_height + 5);
                        len = splits.len();
                        // if there are too many splits, calculate how many and set flag to make a new list to display
                        // otherwise if there are too few and there are enough to display more, set recreate flag
                        if height - timer_height < rows_height {
                            diff = (rows_height - (height - timer_height)) / splits_height;
                            recreate_on_screen = Some(1);
                        } else if rows_height < height - timer_height {
                            diff = ((height - timer_height) - rows_height) / splits_height;
                            if !(max_splits + diff as usize > SPLITS_ON_SCREEN
                                || max_splits + diff as usize > splits.len())
                            {
                                recreate_on_screen = Some(1);
                            }
                        }
                    }

                    // space being used to start, stop, and split for now
                    Event::KeyDown {
                        keycode: Some(Keycode::Space),
                        timestamp: event_time,
                        ..
                    } => match self.state {
                        // if timer isnt started, start it.
                        TimerState::NotStarted { .. } => {
                            total_time = Instant::now();
                            match offset {
                                // if we are in the start offset, tell it to offset
                                Some(x) => {
                                    self.state = TimerState::OffsetCountdown { amt: x };
                                }
                                None => {
                                    self.state = TimerState::Running {
                                        timestamp: event_time,
                                    };
                                }
                            }
                            current_split = 0;
                        }
                        // if it is running, either split or end
                        TimerState::Running { timestamp: t, .. } => {
                            active_run_times.push(u128::from(self.timer.ticks() - split_time_ticks));
                            split_time_ticks = self.timer.ticks();
                            time_str = timing::ms_to_readable(
                                (event_time - t) as u128 + before_pause.unwrap_or(0),
                                true,
                            );
                            text_surface = font.render(&time_str).blended(Color::WHITE).unwrap();
                            texture = creator.create_texture_from_surface(&text_surface).unwrap();
                            split_times[current_split] = texture;
                            if current_split < splits.len() - 1 {
                                current_split += 1;
                            } else {
                                self.state = TimerState::Finished { time_str };
                            }
                            if current_split + 1 > bottom_split_index {
                                bottom_split_index += 1;
                                recreate_on_screen = Some(2);
                                if (event_time - t) as u128 + before_pause.unwrap_or(0) < self.run.pb {
					self.run.pb = (event_time - t) as u128 + before_pause.unwrap_or(0);
					self.run.best_times = active_run_times;
					active_run_times = vec![];
					self.run.save("run.msf");
                                }
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            window_width = self.canvas.viewport().width();
            let mut on_screen_times: &[Texture] = &[];
            // recreate texture slice to display based on flags set earlier
            match recreate_on_screen {
                // set at the start, creates the initial set of splits
                Some(0) => {
                    on_screen = &splits[0..bottom_split_index];
                    on_screen_times = &split_times[0..bottom_split_index];
                    unsafe { recreate_on_screen = RECREATE_DEFAULT };
                }
                // set on window resize, creates new slices based on diff and number of splits
                Some(1) => {
                    if max_splits > diff as usize {
                        max_splits -= diff as usize;
                        if current_split + max_splits > len {
                            bottom_split_index = len;
                            on_screen = &splits[len - max_splits..bottom_split_index];
                            on_screen_times = &split_times[len - max_splits..bottom_split_index];
                        } else if current_split < max_splits {
                            bottom_split_index = max_splits;
                            on_screen = &splits[0..max_splits];
                            on_screen_times = &split_times[0..max_splits];
                        } else if current_split >= max_splits {
                            bottom_split_index = current_split + max_splits;
                            on_screen = &splits[current_split..current_split + max_splits];
                            on_screen_times =
                                &split_times[current_split..current_split + max_splits];
                        }
                    }
                    unsafe { recreate_on_screen = RECREATE_DEFAULT }
                }
                // set on mouse scroll, creates new slices based on the current top and bottom split
                Some(2) => {
                    index = bottom_split_index - max_splits;
                    on_screen = &splits[index..bottom_split_index];
                    on_screen_times = &split_times[index..bottom_split_index];
                    unsafe { recreate_on_screen = RECREATE_DEFAULT }
                }
                // default if there are >0 total splits, recreates times every loop to dodge lifetime problems
                Some(3) => {
                    index = bottom_split_index - max_splits;
                    on_screen_times = &split_times[index..bottom_split_index];
                }
                _ => {}
            }
            if let TimerState::Running { timestamp } = self.state {
                // calculates if run is ahead/behind/gaining/losing and adjusts accordingly
                let ticks = self.timer.ticks();
                if u128::from(ticks - timestamp) + before_pause.unwrap_or(0)
                    < summed_times[current_split]
                {
                    if u128::from(ticks - split_time_ticks) < split_times_ms[current_split] {
                        color = Color::GREEN;
                    } else {
                        color = LOSING_TIME;
                    }
                } else {
                    if u128::from(ticks - split_time_ticks) < split_times_ms[current_split] {
                        color = MAKING_UP_TIME;
                    } else {
                        color = Color::RED;
                    }
                }
                if current_split >= bottom_split_index - 1 {
                    cur = max_splits - 1;
                } else {
                    cur = current_split;
                }
            } else {
                cur = usize::MAX;
                color = Color::WHITE;
            }
            render::render_rows(
                &on_screen,
                &on_screen_times,
                &mut self.canvas,
                window_width,
                cur,
            );
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
                    one_sixtieth - Instant::now().duration_since(frame_time),
                );
            }
        }
    }
    // updates time string based on timer state, basically leaves it the same if timer is paused
    fn update_time(&self, before_pause: Option<u128>, total_time: Instant) -> String {
        let time: String;
        match &self.state {
            TimerState::Running { .. } => {
                time = timing::ms_to_readable(
                    total_time.elapsed().as_millis() + before_pause.unwrap_or(0),
                    false,
                );
            }
            TimerState::Paused {
                time_str: display, ..
            } => {
                time = display.to_string();
            }
            TimerState::Finished { time_str: string }
            | TimerState::NotStarted { time_str: string } => {
                time = string.to_owned();
            }
            TimerState::OffsetCountdown { amt: amount } => {
                if amount > &total_time.elapsed().as_millis() {
                    let num =
                        timing::ms_to_readable(amount - total_time.elapsed().as_millis(), false);
                    time = format!("-{}", num);
                } else {
                    time = "0.000".to_string();
                }
            }
        }
        return time;
    }
}
