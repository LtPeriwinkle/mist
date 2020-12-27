// app struct and its functions, one of which is the application mainloop
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
use crate::config::{self, Config};
use crate::render;
use crate::splits::{self, Run, Split};
use crate::timing;
// struct that holds information about the running app and its state
#[allow(dead_code)]
pub struct App {
    context: sdl2::Sdl,
    ev_pump: sdl2::EventPump,
    timer: Instant,
    canvas: WindowCanvas,
    ttf: sdl2::ttf::Sdl2TtfContext,
    state: TimerState,
    run: splits::Run,
    config: config::Config,
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
        let timer = Instant::now();
        let config: Config;
        match open_file("(OPTIONAL) Open config file", "*.cfg") {
            Some(x) => {
                config = Config::from_file(Some(&x));
            }
            None => {
                config = Config::from_file(None);
                info_dialog(
                    "",
                    "Since you did not select a config file, the default will be used",
                );
            }
        }
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
            config,
        }
    }

    pub fn run(&mut self) {
        let mut path: Option<String> = None;
        let retry: bool;
        if let Some(x) = self.config.file() {
            path = Some(x.to_owned());
            match Run::from_file(&x) {
                Some(r) => {
                    self.run = r;
                    retry = false;
                }
                None => {
                    retry = true
                }
            }
        } else {
            retry = true;
        }
        if retry {
            loop {
                path = open_file("Open split file", "*.msf");
                match path {
                    None => {
                        return;
                    }
                    Some(ref p) => match Run::from_file(&p) {
                        Some(x) => {
                            self.run = x;
                            break;
                        }
                        None => {
                            if !bad_file_dialog("Split file parse failed. Try another file?") {
                                return;
                            }
                        }
                    },
                }
            }
        }
        if let Some(x) = self.config.file() {
		path = Some(x.to_owned());
        }
        let path = path.unwrap();
        self.config.set_file(&path);
        // set up some stuff that's a pain to do elsewhere
        self.canvas.clear();
        let sizes = self.config.fsize();
        let timer_font = self
            .ttf
            .load_font(self.config.tfont(), sizes.0)
            .expect("could not open font file");
        let font = self
            .ttf
            .load_font(self.config.sfont(), sizes.1)
            .expect("could not open font file");
        let creator = self.canvas.texture_creator();
        let timer_height = timer_font.size_of("0123456789").unwrap().1;
        let splits_height = font
            .size_of("qwertyuiopasdfghjklzxcvbnm01234567890")
            .unwrap()
            .1;
        // set the minimum height of the window to the size of the time texture
        self.canvas
            .window_mut()
            .set_minimum_size(0, timer_height + 10)
            .unwrap();

        // get first vec of split name textures from file
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
        let split_times_ms: Vec<u128> = self.run.get_times().iter().cloned().collect();
        let mut summed_times = timing::split_time_sum(&split_times_ms);
        let split_times_raw: Vec<String> = summed_times
            .iter()
            .map(|val| timing::split_time_text(*val))
            .collect();
        // initialize variables that are used in the loop for replacing timer texture
        let mut text_surface: Surface;
        let mut texture: Texture;
        // vectors that hold the textures for split names and their associated times
        let mut splits: Vec<Split> = vec![];

        let mut index = 0;
        // convert the split names into textures and add them to the split name vec
        while index < split_names.len() {
            let text_surface = font
                .render(&split_names[index])
                .blended(Color::WHITE)
                .expect("split name render failed");
            let texture = creator
                .create_texture_from_surface(&text_surface)
                .expect("split name texture failed");
            let pb = font
                .render(&split_times_raw[index])
                .blended(Color::WHITE)
                .expect("split time render failed");
            let pb_texture = creator
                .create_texture_from_surface(&pb)
                .expect("split time texture failed");
            let split = Split::new(
                split_times_ms[index],
                self.run.gold_time(index),
                0,
                None,
                texture,
                pb_texture,
                None,
            );
            splits.push(split);
            index += 1;
        }

        // set up max splits dynamically in case there are too few splits
        let mut max_splits: usize;
        let mut bottom_split_index = SPLITS_ON_SCREEN;
        let mut top_split_index = 0;
        if SPLITS_ON_SCREEN > split_names.len() {
            bottom_split_index = split_names.len();
            max_splits = split_names.len();
        } else {
            max_splits = SPLITS_ON_SCREEN;
        }
        // drop stuff that isnt needed after initializing
        drop(split_times_ms);
        drop(split_times_raw);
        drop(split_names);
        // set up variables used in the mainloop
        // framerate cap timer
        let mut frame_time: Instant;
        let mut total_time = Instant::now();
        // display time
        let mut time_str: String;
        // keeps track of whether timer has been paused and paused value
        let mut before_pause = 0;
        let mut before_pause_split = 0;
        // this one should be a static but duration isnt allowed to be static apparently
        let one_sixtieth = Duration::new(0, 1_000_000_000 / 60);
        let mut current_split = 0;
        // these two to avoid having to drop and reallocate every loop
        let mut window_width: u32;
        let mut color = Color::WHITE;
        // sum of split times for display on rows
        let mut recreate_on_screen: Option<u8> = Some(0);
        // diff between max on screen and current, used when resizing window
        let mut diff: u32 = 0;
        let len: usize = splits.len();
        // current split in the slice of splits sent to render_time()
        let mut cur: usize;
        // elapsed time when last split happened
        let mut split_time_ticks = 0;
        // split times of current run
        let mut active_run_times: Vec<u128> = vec![];

        let mut elapsed: u128;
        let mut save = false;
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
                    elapsed = self.timer.elapsed().as_millis();
                    self.state = TimerState::Running { timestamp: elapsed };
                    split_time_ticks = elapsed;
                    before_pause_split = 0;
                    total_time = Instant::now();
                }
            }
            if save {
                save = false;
                if save_check() {
                    self.run.save(&path);
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
                        repeat: false,
                        ..
                    } => {
                        elapsed = self.timer.elapsed().as_millis();
                        match self.state {
                            // if timer is paused, unpause it, put the amount of time before the pause in a variable
                            // and set the state to running
                            TimerState::Paused {
                                time: t, split: s, ..
                            } => {
                                total_time = Instant::now();
                                split_time_ticks = elapsed;
                                before_pause = t;
                                before_pause_split = s;
                                self.state = TimerState::Running { timestamp: elapsed };
                            }
                            // if the timer is already running, set it to paused.
                            TimerState::Running { .. } => {
                                self.state = TimerState::Paused {
                                    time: total_time.elapsed().as_millis() + before_pause,
                                    split: (elapsed - split_time_ticks) + before_pause_split,
                                    time_str: timing::ms_to_readable(
                                        total_time.elapsed().as_millis() + before_pause,
                                        true,
                                    ),
                                };
                            }
                            _ => {}
                        }
                    }

                    // R key to reset timer
                    Event::KeyDown {
                        keycode: Some(Keycode::R),
                        repeat: false,
                        ..
                    } => {
                        active_run_times = vec![];
                        bottom_split_index = max_splits;
                        recreate_on_screen = Some(2);
                        match offset {
                            // if there is an offset, reset the timer to that, if not, reset timer to 0
                            Some(x) => {
                                before_pause = 0;
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
                        index = 0;
                        while index < len {
                            splits[index].set_cur(None);
                            splits[index].set_diff(0, None);
                            index += 1;
                        }
                    }

                    // handle vertical window resize by changing number of splits
                    Event::Window {
                        win_event: WindowEvent::Resized(..),
                        ..
                    } => {
                        let height = self.canvas.viewport().height();
                        let rows_height = max_splits as u32 * (splits_height + 5);
                        // if there are too many splits, calculate how many and set flag to make a new list to display
                        // otherwise if there are too few and there are enough to display more, set recreate flag
                        if height - timer_height < rows_height {
                            diff = (rows_height - (height - timer_height)) / splits_height;
                            recreate_on_screen = Some(1);
                        } else if rows_height < height - timer_height {
                            diff = ((height - timer_height) - rows_height) / splits_height;
                            if !(max_splits + diff as usize > SPLITS_ON_SCREEN
                                || max_splits + diff as usize > len)
                            {
                                recreate_on_screen = Some(1);
                            }
                        }
                    }

                    // space being used to start, stop, and split for now
                    Event::KeyDown {
                        keycode: Some(Keycode::Space),
                        repeat: false,
                        ..
                    } => match self.state {
                        // if timer isnt started, start it.
                        TimerState::NotStarted { .. } => {
                            elapsed = self.timer.elapsed().as_millis();
                            total_time = Instant::now();
                            match offset {
                                // if we are in the start offset, tell it to offset
                                Some(x) => {
                                    self.state = TimerState::OffsetCountdown { amt: x };
                                }
                                None => {
                                    self.state = TimerState::Running { timestamp: elapsed };
                                }
                            }
                            current_split = 0;
                        }
                        // if it is running, either split or end
                        TimerState::Running { timestamp: t, .. } => {
                            if len != 0 {
                            elapsed = self.timer.elapsed().as_millis();
                            active_run_times
                                .push((elapsed - split_time_ticks) + before_pause_split);
                            split_time_ticks = elapsed;
                            let sum = timing::split_time_sum(&active_run_times)[current_split];
                            let diff = sum as i128 - summed_times[current_split] as i128;
                            time_str = timing::diff_text(diff);
                            if active_run_times[current_split] < splits[current_split].gold() {
                                color = GOLD;
                                splits[current_split].set_gold(active_run_times[current_split]);
                            }
                            text_surface = font.render(&time_str).blended(color).unwrap();
                            texture = creator.create_texture_from_surface(&text_surface).unwrap();
                            splits[current_split].set_diff(diff, Some(texture));
                            time_str = timing::split_time_text((elapsed - t) + before_pause);
                            text_surface = font.render(&time_str).blended(Color::WHITE).unwrap();
                            texture = creator.create_texture_from_surface(&text_surface).unwrap();
                            splits[current_split].set_cur(Some(texture));
                            if current_split < splits.len() - 1 {
                                current_split += 1;
                            } else {
                                let mut index = 0;
                                while index < len {
                                    if splits[index].gold() < self.run.gold_time(index) {
                                        self.run.set_gold_time(index, splits[index].gold());
                                    }
                                }
                                self.state = TimerState::Finished {
                                    time_str: timing::ms_to_readable(
                                        (elapsed - t) + before_pause,
                                        true,
                                    ),
                                };
                                if (elapsed - t) + before_pause < self.run.pb() {
                                    index = 0;
                                    summed_times = timing::split_time_sum(&active_run_times);
                                    let split_times_raw: Vec<String> = summed_times
                                        .iter()
                                        .map(|val| timing::split_time_text(*val))
                                        .collect();
                                    while index < len {
                                        text_surface = font
                                            .render(&split_times_raw[index])
                                            .blended(Color::WHITE)
                                            .unwrap();
                                        texture = creator
                                            .create_texture_from_surface(text_surface)
                                            .unwrap();
                                        splits[index].set_pb(texture);
                                        splits[index].set_cur(None);
                                        splits[index].set_time(active_run_times[index]);
                                    }
                                    save = true;
                                    self.run.set_pb((elapsed - t) + before_pause);
                                    self.run.set_times(&active_run_times);
                                    active_run_times = vec![];
                                }
                            }
                            if current_split + 1 > bottom_split_index {
                                bottom_split_index += 1;
                                recreate_on_screen = Some(2);
                            }
                            } else {
                            	elapsed = self.timer.elapsed().as_millis();
				self.state = TimerState::Finished {time_str: timing::ms_to_readable((elapsed - t) + before_pause, true)};
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            window_width = self.canvas.viewport().width();

            // make some changes to stuff before updating screen based on what happened in past loop
            if let TimerState::Running { .. } = self.state {
                // calculates if run is ahead/behind/gaining/losing and adjusts accordingly
                elapsed = self.timer.elapsed().as_millis();
                if current_split == 0 && len != 0{
                    if (elapsed - split_time_ticks) + before_pause_split
                        < splits[current_split].time()
                    {
                        color = AHEAD;
                    } else {
                        color = BEHIND;
                    }
                } else if len != 0 {
                    let allowed =
                        splits[current_split].time() as i128 - splits[current_split - 1].diff();
                    let buffer = splits[current_split - 1].diff();
                    let time = ((elapsed - split_time_ticks) + before_pause_split) as i128;
                    if buffer < 0 {
                        if time > allowed {
                            color = BEHIND;
                        } else if time < allowed && time > allowed + buffer {
                            color = LOSING_TIME;
                        } else {
                            color = AHEAD;
                        }
                    } else {
                        if time > allowed && time < allowed + buffer {
                            color = MAKING_UP_TIME;
                        } else if time > allowed && time > allowed + buffer {
                            color = BEHIND;
                        } else {
                            color = AHEAD;
                        }
                    }
                }
                if current_split >= top_split_index && current_split <= bottom_split_index {
                    cur = current_split - top_split_index;
                } else {
                    cur = usize::MAX;
                }
            } else {
                cur = usize::MAX;
                color = Color::WHITE;
            }
            // change top and bottom split indices based on flags set earlier in loop
            match recreate_on_screen.take() {
                // set at the start, creates the initial set of splits
                Some(0) => {
                    top_split_index = 0;
                }
                // set on window resize, creates new slices based on diff and number of splits
                Some(1) => {
                    if max_splits > diff as usize {
                        max_splits -= diff as usize;
                        if current_split + max_splits > len {
                            bottom_split_index = len;
                            top_split_index = len - max_splits;
                        } else if current_split < max_splits {
                            bottom_split_index = max_splits;
                            top_split_index = 0;
                        } else if current_split >= max_splits {
                            bottom_split_index = current_split + max_splits;
                            top_split_index = current_split;
                        }
                    }
                }
                // set on mouse scroll, creates new slices based on the current top and bottom split
                Some(2) => {
                    top_split_index = bottom_split_index - max_splits;
                }
                _ => {}
            }
            render::render_rows(
                &splits[top_split_index..bottom_split_index],
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
        self.config.save(None);
    }
    // updates time string based on timer state, basically leaves it the same if timer is paused
    fn update_time(&self, before_pause: u128, total_time: Instant) -> String {
        let time: String;
        match &self.state {
            TimerState::Running { .. } => {
                time =
                    timing::ms_to_readable(total_time.elapsed().as_millis() + before_pause, false);
            }
            TimerState::Finished { time_str: string }
            | TimerState::NotStarted { time_str: string }
            | TimerState::Paused {
                time_str: string, ..
            } => {
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
