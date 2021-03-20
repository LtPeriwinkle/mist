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

use mist_run_utils::run::Run;

use crate::comparison::Comparison;
use crate::components::*;
use crate::config::{self, Config};
use crate::render;
use crate::splits::Split;
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
    comparison: Comparison,
    run: Run,
    config: config::Config,
}

impl App {
    pub fn init(context: sdl2::Sdl) -> Self {
        // sdl setup boilerplate
        // there are a lot of errors that are improperly handled here; i'll get around to it eventually but its not my top priority
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
        // start the overarching application timer (kinda)
        let timer = Instant::now();
        // return an App that hasn't started and has an empty run
        let mut app = App {
            context,
            ev_pump,
            timer,
            canvas,
            ttf,
            state: TimerState::NotStarted {
                time_str: "0.000".to_string(),
            },
            comparison: Comparison::PersonalBest,
            run: Run::default(),
            config: Config::open(),
        };
        let mut path: Option<String> = None;
        let retry: bool;
        // try to use the filepath specified in the config file
        // if it doesnt exist then set retry to true
        // if the specified file is invalid also set retry
        if let Some(x) = app.config.file() {
            path = Some(x.to_owned());
            match Run::from_msf_file(&x) {
                Some(r) => {
                    app.run = r;
                    retry = false;
                }
                None => retry = true,
            }
        } else {
            retry = true;
        }
        // if retry was set earlier then enter the loop of picking a file
        if retry {
            loop {
                // open a a file choosing dialog box
                path = open_file("Open split file", "*.msf");
                // if the user didn't pick a file and hit cancel, then exit this function (which currently will exit the program)
                match path {
                    None => {
                        std::process::exit(0);
                    }
                    // if the user did choose a file, try to parse a Run from it.
                    // if the run is valid, break the file loop and continue on
                    Some(ref p) => match Run::from_msf_file(&p) {
                        Some(x) => {
                            app.run = x;
                            break;
                        }
                        None => {
                            // if it is invalid, ask the user whether they want to try another file
                            // if they don't then exit the program
                            if !bad_file_dialog("Split file parse failed. Try another file?") {
                                std::process::exit(0)
                            }
                        }
                    },
                }
            }
        }
        // remove Option wrapper from filepath for later use since it now is guaranteed not to be None
        let mut path = path.unwrap();
        // set the config file's run path to the given path in case a new one was chosen
        app.config.set_file(&path);
        return app;
    }

    pub fn run(&mut self) {
        let mut path = self.config.file().unwrap().to_string();

        self.canvas.clear();

        let colors = self.config.color_list();
        let ahead = Color::from(colors[0]);
        let behind = Color::from(colors[1]);
        let making_up_time = Color::from(colors[2]);
        let losing_time = Color::from(colors[3]);
        let gold = Color::from(colors[4]);

        // grab font sizes from config file and load the fonts
        let sizes = self.config.fsize();
        let mut timer_font = self
            .ttf
            .load_font(self.config.tfont(), sizes.0)
            .expect("could not open font file");
        timer_font.set_kerning(false);
        let font = self
            .ttf
            .load_font(self.config.sfont(), sizes.1)
            .expect("could not open font file");
        // make the texture creator used a lot later on
        let creator = self.canvas.texture_creator();

        // get the heights of different font textures
        let splits_height = font
            .size_of("qwertyuiopasdfghjklzxcvbnm01234567890!@#$%^&*(){}[]|\\:;'\",.<>?/`~-_=+")
            .unwrap()
            .1;
        let timer_height = timer_font.size_of("-0123456789:.").unwrap().1 + (splits_height - 1);
        // set the minimum height of the window to the size of the time texture
        self.canvas
            .window_mut()
            .set_minimum_size(0, timer_height + 20)
            .unwrap();

        // get first vec of split name textures from file
        let split_names = self.run.split_names();
        let mut offset = self.run.offset();
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
            let comp = font
                .render(&split_times_raw[index])
                .blended(Color::WHITE)
                .expect("split time render failed");
            let comp_texture = creator
                .create_texture_from_surface(&comp)
                .expect("split time texture failed");
            // create split struct with its corresponding times and textures
            let split = Split::new(
                split_times_ms[index],
                self.run.gold_time(index),
                0,
                None,
                texture,
                comp_texture,
                None,
            );
            splits.push(split);
            index += 1;
        }

        let mut max_splits: usize;
        let mut bottom_split_index: usize;
        let mut top_split_index = 0;

        // if there are too few splits then set the max splits to the number of splits rather than
        // the max allowed amount
        let max_initial_splits: usize = ((500 - timer_height) / splits_height) as usize;
        if max_initial_splits > split_names.len() {
            bottom_split_index = split_names.len();
            max_splits = split_names.len();
        } else {
            max_splits = max_initial_splits;
            bottom_split_index = max_initial_splits;
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
        let mut len: usize = splits.len();
        // current split in the slice of splits sent to render_time()
        let mut cur: usize;
        // elapsed time when last split happened
        let mut split_time_ticks = 0;
        // split times of current run
        let mut active_run_times: Vec<u128> = vec![];

        // variable used to hold elapsed milliseconds of the application timer
        let mut elapsed: u128;
        // set when a run ends and is a pb to signal for a pop-up window to ask if the user wants to save
        let mut save = false;

        let mut comp_changed = false;
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
            // check at the start of the loop (after rendering final frame of an ended run)
            // if the user wants to save the run or not with a yes/no popup window

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
                        if top_split_index != 0 {
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
                        // reset active run times and return the list of splits to the top
                        active_run_times = vec![];
                        bottom_split_index = max_splits;
                        before_pause = 0;
                        before_pause_split = 0;
                        color = ahead;
                        match offset {
                            // if there is an offset, reset the timer to that, if not, reset timer to 0
                            Some(x) => {
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
                        // get rid of run-specific active times and differences
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
                        let rows_height =
                            ((bottom_split_index - top_split_index) as u32 * (splits_height + 2)) + splits_height;
                        // if there are too many splits, calculate how many and set flag to make a new list to display
                        // otherwise if there are too few and there are enough to display more, set recreate flag
                        if height - timer_height < rows_height {
                            diff = (rows_height - (height - timer_height)) / splits_height;
                            recreate_on_screen = Some(1);
                        } else if rows_height < height - timer_height {
                            diff = ((height - timer_height) - rows_height) / splits_height;
                            if !(max_splits + diff as usize > max_initial_splits
                                || max_splits + diff as usize > len)
                            {
                                recreate_on_screen = Some(3);
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
                            split_time_ticks = elapsed;
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
                            // only try to do this stuff if there is at least one split
                            if len != 0 {
                                elapsed = self.timer.elapsed().as_millis();
                                active_run_times
                                    .push((elapsed - split_time_ticks) + before_pause_split);
                                split_time_ticks = elapsed;
                                before_pause_split = 0;
                                let sum = timing::split_time_sum(&active_run_times)[current_split];
                                let diff = sum as i128 - summed_times[current_split] as i128;
                                time_str = timing::diff_text(diff);
                                // set diff color to gold and replace split gold
                                if active_run_times[current_split] < splits[current_split].gold() {
                                    save = true;
                                    color = gold;
                                    self.run.set_gold_time(current_split, active_run_times[current_split]);
                                    splits[current_split].set_gold(active_run_times[current_split]);
                                }
                                text_surface = font.render(&time_str).blended(color).unwrap();
                                texture =
                                    creator.create_texture_from_surface(&text_surface).unwrap();
                                splits[current_split].set_diff(diff, Some(texture));
                                time_str = timing::split_time_text((elapsed - t) + before_pause);
                                text_surface =
                                    font.render(&time_str).blended(Color::WHITE).unwrap();
                                texture =
                                    creator.create_texture_from_surface(&text_surface).unwrap();
                                splits[current_split].set_cur(Some(texture));
                                // if there are still splits left, continue the run and advance the current split
                                if current_split < splits.len() - 1 {
                                    current_split += 1;
                                // otherwise end the run
                                } else {
                                    self.state = TimerState::Finished {
                                        time_str: timing::ms_to_readable(
                                            (elapsed - t) + before_pause,
                                            true,
                                        ),
                                    };
                                    // if this run was a pb then set the Run struct's pb and splits
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
                                            splits[index].set_comp_tex(texture);
                                            splits[index].set_cur(None);
                                            splits[index].set_time(active_run_times[index]);
                                            index += 1;
                                        }
                                        save = true;
                                        self.run.set_pb((elapsed - t) + before_pause);
                                        self.run.set_times(&active_run_times);
                                        active_run_times = vec![];
                                    }
                                }
                                // if the next split is offscreen set recreate_on_screen flag to change the current split slice
                                if current_split + 1 > bottom_split_index {
                                    bottom_split_index += 1;
                                    recreate_on_screen = Some(2);
                                }
                            // finish the run if there are no splits
                            } else {
                                elapsed = self.timer.elapsed().as_millis();
                                self.state = TimerState::Finished {
                                    time_str: timing::ms_to_readable(
                                        (elapsed - t) + before_pause,
                                        true,
                                    ),
                                };
                            }
                        }
                        _ => {}
                    },
                    // right and left arrows to swap comparisons
                    Event::KeyDown {
                        keycode: Some(Keycode::Right),
                        ..
                    } => {
                        self.comparison.next();
                        comp_changed = true;
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Left),
                        ..
                    } => {
                        self.comparison.prev();
                        comp_changed = true;
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::F1),
                        ..
                    } => {
                        if let TimerState::NotStarted { .. } = self.state {
                            if save {
                                if save_check() {
                                    self.run.save_msf(&path);
                                }
                            }
                            match reload_splits() {
                                Some((r, p)) => {
                                    self.run = r;
                                    path = p;
                                }
                                _ => {}
                            }
                            offset = self.run.offset();
                            // if there is an offset, display it properly
                            match offset {
                                Some(x) => {
                                    self.state = TimerState::NotStarted {
                                        time_str: format!("-{}", timing::ms_to_readable(x, false)),
                                    };
                                }
                                _ => {}
                            }
                            let split_names = self.run.split_names();
                            let split_times_ms: Vec<u128> =
                                self.run.get_times().iter().cloned().collect();
                            summed_times = timing::split_time_sum(&split_times_ms);
                            let split_times_raw: Vec<String> = summed_times
                                .iter()
                                .map(|val| timing::split_time_text(*val))
                                .collect();
                            splits = vec![];
                            index = 0;
                            while index < split_names.len() {
                                let text_surface = font
                                    .render(&split_names[index])
                                    .blended(Color::WHITE)
                                    .expect("split name render failed");
                                let texture = creator
                                    .create_texture_from_surface(&text_surface)
                                    .expect("split name texture failed");
                                let comp = font
                                    .render(&split_times_raw[index])
                                    .blended(Color::WHITE)
                                    .expect("split time render failed");
                                let comp_texture = creator
                                    .create_texture_from_surface(&comp)
                                    .expect("split time texture failed");
                                let split = Split::new(
                                    split_times_ms[index],
                                    self.run.gold_time(index),
                                    0,
                                    None,
                                    texture,
                                    comp_texture,
                                    None,
                                );
                                splits.push(split);
                                index += 1;
                            }
                            if len == 0 {
				max_splits = ((self.canvas.viewport().height() - timer_height) / splits_height) as usize;
                            }
                            len = splits.len();
                            if max_splits > len {
                                max_splits = len;
                            }
                            top_split_index = 0;
                            bottom_split_index = max_splits;
                        }
                    }
                    _ => {}
                }
            }
            if comp_changed {
                comp_changed = false;
                let mut index = 0;
                if let Comparison::None = self.comparison {
                    while index < len {
                        let surface = font
                            .render("-  ")
                            .blended(Color::WHITE)
                            .expect("render failed");
                        let tex = creator
                            .create_texture_from_surface(&surface)
                            .expect("surface failed");
                        splits[index].set_comp_tex(tex);
                        index += 1;
                    }
                } else {
                    let split_times = match self.comparison {
                        Comparison::PersonalBest => self.run.get_times().to_vec(),
                        Comparison::Golds => self.run.get_golds().to_vec(),
                        _ => {
                            vec![]
                        }
                    };
                    let split_times_raw: Vec<String> = timing::split_time_sum(&split_times)
                        .iter()
                        .map(|val| timing::split_time_text(*val))
                        .collect();
                    while index < len {
                        let surface = font
                            .render(&split_times_raw[index])
                            .blended(Color::WHITE)
                            .expect("render failed");
                        let tex = creator
                            .create_texture_from_surface(&surface)
                            .expect("surface failed");
                        splits[index].set_comp_tex(tex);
                        index += 1;
                    }
                }
            }

            window_width = self.canvas.viewport().width();

            // make some changes to stuff before updating screen based on what happened in past loop
            // but only if the timer is running
            if let TimerState::Running { .. } = self.state {
                // calculates if run is ahead/behind/gaining/losing and adjusts accordingly
                elapsed = self.timer.elapsed().as_millis();
                // if we are in split 0 there's no need for fancy losing/gaining time, only ahead and behind
                if current_split == 0 && len != 0 {
                    if (elapsed - split_time_ticks) + before_pause_split
                        < splits[current_split].time()
                    {
                        color = ahead;
                    } else {
                        color = behind;
                    }
                } else if len != 0 {
                    if let Comparison::None = self.comparison {
                        color = Color::WHITE;
                    } else {
                        // get the amount of time that the runner could spend on the split without being behind comparison
                        let allowed: i128;
                        match self.comparison {
                            Comparison::PersonalBest => {
                                allowed = splits[current_split].time() as i128
                                    - splits[current_split - 1].diff();
                            }
                            Comparison::Golds => {
                                allowed = splits[current_split].gold() as i128
                                    - splits[current_split - 1].diff();
                            }
                            _ => {
                                allowed = 0;
                            }
                        }
                        let buffer = splits[current_split - 1].diff();
                        // get amount of time that has passed in the current split
                        let time = ((elapsed - split_time_ticks) + before_pause_split) as i128;
                        // if the last split was ahead of comparison split
                        if buffer < 0 {
                            // if the runner has spent more time than allowed they have to be behind
                            if time > allowed {
                                color = behind;
                            // if they have spent less than the time it would take to become behind but more time than they took in the pb,
                            // then they are losing time but still ahead. default color for this is lightish green like LiveSplit
                            } else if time < allowed && time > allowed + buffer {
                                color = losing_time;
                            // if neither of those are true the runner must be ahead
                            } else {
                                color = ahead;
                            }
                        // if last split was behind comparison split
                        } else {
                            // if the runner has gone over the amount of time they should take but are still on better pace than
                            // last split then they are making up time. a sort of light red color like livesplit
                            if time > allowed && time < allowed + buffer {
                                color = making_up_time;
                            // if they are behind both the allowed time and their current pace they must be behind
                            } else if time > allowed && time > allowed + buffer {
                                color = behind;
                            // even if the last split was behind, often during part of the split the runner could finish it and come out ahead
                            } else {
                                color = ahead;
                            }
                        }
                    }
                }
                // set the split to highlight in blue when rendering
                // this value has to be adjusted to be relative to the number of splits on screen rather than
                // the total number of splits
                if current_split >= top_split_index && current_split <= bottom_split_index {
                    cur = current_split - top_split_index;
                } else {
                    // if the current split isnt on screen, pass this horrendously massive value to the render function
                    // so that it doesnt put a blue rectangle on anything (hopefully)
                    cur = usize::MAX;
                }
            // if timer isnt running then dont highlight a split or use a color
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
                // set on mouse scroll or on split if the next split is offscreen
                // creates new slices based on the current top and bottom split
                Some(2) => {
                    top_split_index = bottom_split_index - max_splits;
                }
                // similar to Some(1) except set when window grows instead of shrinks
                Some(3) => {
                    max_splits += diff as usize;
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
                _ => {}
            }
            // copy the name, diff, and time textures to the canvas
            // and highlight the split relative to the top of the list marked by cur
            // function places the rows and ensures that they don't go offscreen
            render::render_rows(
                &splits[top_split_index..bottom_split_index],
                &mut self.canvas,
                window_width,
                cur,
            );
            // update the time based on the current timer state
            time_str = self.update_time(before_pause, total_time);
            text_surface = timer_font
                .render(&time_str)
                .shaded(color, Color::BLACK)
                .expect("time font render failed");
            texture = creator
                .create_texture_from_surface(&text_surface)
                .expect("time texture creation failed");
            // copy the time texture to the canvas. function takes care of placing and making sure it doesnt try to place the texture offscreen
            render::render_time(&texture, &mut self.canvas);
            self.canvas.present();
            if Instant::now().duration_since(frame_time) <= one_sixtieth {
                thread::sleep(
                    // if the entire loop pass was completed in under 1/60 second, delay to keep the framerate at ~60fps
                    one_sixtieth - Instant::now().duration_since(frame_time),
                );
            }
        }
        // after the loop is exited then save the config file
        self.config.save();
        if save {
            if save_check() {
                self.run.save_msf(&path);
            }
        }
    }
    // updates time string based on timer state, basically leaves it the same if timer is not running
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
