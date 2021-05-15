// app struct and its functions, one of which is the application mainloop
use sdl2::event::{Event, WindowEvent};
#[cfg(feature = "bg")]
use sdl2::gfx::rotozoom::RotozoomSurface;
#[cfg(any(feature = "icon", feature = "bg"))]
use sdl2::image::LoadSurface;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
#[cfg(feature = "bg")]
use sdl2::pixels::PixelFormatEnum;
#[cfg(feature = "bg")]
use sdl2::rect::Rect;
#[cfg(feature = "bg")]
use sdl2::render::TextureAccess;
use sdl2::render::{Texture, WindowCanvas};
use sdl2::surface::Surface;
use sdl2::ttf;

use std::thread;
use std::time::{Duration, Instant};

use mist_run_utils::run::Run;

use crate::comparison::Comparison;
use crate::components::*;
use crate::config::{self, Config};
use crate::error;
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
        let video = context.video().unwrap_or_else(|err| {
            error!("video context failure: {}", err);
        });
        let mut window = video
            .window("mist", 300, 500)
            .position_centered()
            .resizable()
            .build()
            .unwrap_or_else(|err| {
                error!("window build failure: {} ", err);
            });
        #[cfg(feature = "icon")]
        {
            let icon = Surface::from_file("assets/MIST.png").unwrap_or_else(|err| {
                error!("couldn't open icon: {}", err);
            });
            window.set_icon(icon);
        }
        let canvas = window.into_canvas().build().unwrap_or_else(|err| {
            error!("couldn't create canvas: {}", err);
        });
        let ttf = ttf::init().unwrap_or_else(|err| {
            error!("ttf init failure: {} ", err);
        });
        let ev_pump = context.event_pump().unwrap_or_else(|err| {
            error!("couldn't create evpump: {}", err);
        });
        // start the overarching application timer (kinda)
        let timer = Instant::now();
        // return an App that hasn't started and has an empty run
        let mut app = App {
            context,
            ev_pump,
            timer,
            canvas,
            ttf,
            state: TimerState::NotRunning {
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
        let path = path.unwrap();
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
        let bg_color = Color::from(colors[5]);

        // grab font sizes from config file and load the fonts
        let sizes = self.config.fsize();
        let mut timer_font = self
            .ttf
            .load_font(self.config.tfont(), sizes.0)
            .unwrap_or_else(|err| {
                error!("tfont load failed", err);
            });
        timer_font.set_kerning(false);
        let font = self
            .ttf
            .load_font(self.config.sfont(), sizes.1)
            .unwrap_or_else(|err| {
                error!("couldn't open sfont: {}", err);
            });
        // make the texture creator used a lot later on
        let creator = self.canvas.texture_creator();

        #[cfg(feature = "bg")]
        let has_bg: bool;
        #[cfg(feature = "bg")]
        let bg_tex: Texture;
        #[cfg(feature = "bg")]
        let bg_rect: Rect;
        #[cfg(feature = "bg")]
        {
            let bg: Option<Surface> = match self.config.img() {
                Some(ref p) => Some(Surface::from_file(&p).unwrap_or_else(|err| {
                    error!("couldn't open bg image: {}", err);
                })),
                None => None,
            };
            if let Some(x) = bg {
                has_bg = true;
                let width = self.canvas.viewport().width();
                let height = self.canvas.viewport().height();
                if !self.config.img_scaled() {
                    let mut sur = Surface::new(width, height, PixelFormatEnum::RGB24)
                        .unwrap_or_else(|err| {
                            error!("couldn't create new surface: {}", err);
                        });
                    let cutoffx = {
                        if x.width() > width {
                            ((x.width() - width) / 2) as i32
                        } else {
                            0
                        }
                    };
                    let cutoffy = {
                        if x.height() > height {
                            ((x.height() - height) / 2) as i32
                        } else {
                            0
                        }
                    };
                    x.blit(Rect::new(cutoffx, cutoffy, width, height), &mut sur, None)
                        .unwrap_or_else(|err| {
                            error!("couldn't blit cropped bg: {}", err);
                        });
                    bg_tex = creator
                        .create_texture_from_surface(&sur)
                        .unwrap_or_else(|err| {
                            error!("couldn't create crop bg tex: {}", err);
                        });
                } else {
                    let sur: Surface;
                    if x.width() > x.height() && width < x.width() {
                        if width < x.width() {
                            sur = x
                                .rotozoom(0.0, width as f64 / x.width() as f64, true)
                                .unwrap_or_else(|err| {
                                    error!("couldn't rotozoom: {}", err);
                                });
                        } else {
                            sur = x
                                .rotozoom(0.0, x.width() as f64 / width as f64, true)
                                .unwrap_or_else(|err| {
                                    error!("couldn't rotozoom: {}", err);
                                });
                        }
                    } else {
                        if height < x.height() {
                            sur = x
                                .rotozoom(0.0, height as f64 / x.height() as f64, true)
                                .unwrap_or_else(|err| {
                                    error!("couldn't rotozoom: {}", err);
                                });
                        } else {
                            sur = x
                                .rotozoom(0.0, x.height() as f64 / height as f64, true)
                                .unwrap_or_else(|err| {
                                    error!("couldn't rotozoom: {}", err);
                                });
                        }
                    }
                    bg_tex = creator
                        .create_texture_from_surface(&sur)
                        .unwrap_or_else(|err| {
                            error!("couldn't create scale bx tex: {}", err);
                        });
                }
            } else {
                has_bg = false;
                bg_tex = creator
                    .create_texture(None, TextureAccess::Static, 1, 1)
                    .unwrap_or_else(|err| {
                        error!("empty bg tex failed: {}", err);
                    });
            }
            let sdl2::render::TextureQuery {
                width: bgw,
                height: bgh,
                ..
            } = bg_tex.query();
            bg_rect = Rect::new(0, 0, bgw, bgh);
        }
        // get the heights of different font textures
        let splits_height = font
            .size_of("qwertyuiopasdfghjklzxcvbnm01234567890!@#$%^&*(){}[]|\\:;'\",.<>?/`~-_=+")
            .unwrap_or_else(|err| {
                error!("split height failed: {}", err);
            })
            .1;
        // get the x-coordinates of characters in the font spritemap
        let coords: Vec<u32> = {
            let mut raw: Vec<u32> = vec![];
            let mut ret: Vec<u32> = vec![0];
            for chr in "-0123456789:. ".chars() {
                let size = timer_font.size_of(&chr.to_string()).unwrap_or_else(|err| {
                    error!("tfont char size failed: {}", err);
                });
                raw.push(size.0);
                ret.push(raw.iter().sum::<u32>());
            }
            ret.push(*raw.iter().max().unwrap());

            ret
        };
        let font_y = timer_font
            .size_of("-0123456789:.")
            .unwrap_or_else(|err| {
                error!("tfont height failed: {}", err);
            })
            .1;
        // render initial white font map. gets overwritten when color changes
        let map = timer_font
            .render("- 0 1 2 3 4 5 6 7 8 9 : .")
            .blended(Color::WHITE)
            .unwrap_or_else(|err| {
                error!("map render failed: {}", err);
            });
        let mut map_tex = creator
            .create_texture_from_surface(&map)
            .unwrap_or_else(|err| {
                error!("map tex creation failed: {}", err);
            });
        // set the height where overlap with splits is checked when resizing window
        let timer_height = font_y + splits_height;
        // set the minimum height of the window to the size of the time texture
        self.canvas
            .window_mut()
            .set_minimum_size(0, timer_height + 20)
            .unwrap_or_else(|err| {
                error!("win set size failed: {}", err);
            });

        // get first vec of split name textures from file
        let split_names = self.run.split_names();
        let mut offset = self.run.offset();
        // if there is an offset, display it properly
        match offset {
            Some(x) => {
                self.state = TimerState::NotRunning {
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
        // initialize variables that are used in the loop for replacing textures
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
                .unwrap_or_else(|err| {
                    error!("text sur failed: {}", err);
                });
            let texture = creator
                .create_texture_from_surface(&text_surface)
                .unwrap_or_else(|err| {
                    error!("text tex failed: {}", err);
                });
            let comp = font
                .render(&split_times_raw[index])
                .blended(Color::WHITE)
                .unwrap_or_else(|err| {
                    error!("comparison sur failed: {}", err);
                });
            let comp_texture = creator
                .create_texture_from_surface(&comp)
                .unwrap_or_else(|err| {
                    error!("comparison tex failed: {}", err);
                });
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

        let mut bottom_split_index: usize;
        let mut top_split_index = 0;
        let mut max_splits: usize;

        // if there are too few splits then set the max splits to the number of splits rather than
        // the max allowed amount
        let max_initial_splits: usize = ((500 - timer_height) / splits_height) as usize;
        if max_initial_splits > splits.len() {
            bottom_split_index = splits.len() - 1;
            max_splits = splits.len();
        } else {
            max_splits = max_initial_splits;
            bottom_split_index = max_initial_splits - 1;
        }
        // drop stuff that isnt needed after initializing
        drop(split_times_ms);
        drop(split_times_raw);
        drop(split_names);
        drop(map);

        // set up variables used in the mainloop
        // framerate cap timer
        let mut frame_time: Instant;
        // instant used to calculate time that the timer has been running
        let mut total_time = Instant::now();
        // display time
        let mut time_str: String;
        // keep track of amount of time that passed before the timer was paused
        let mut before_pause = 0;
        let mut before_pause_split = 0;
        // this one should be a static but duration isnt allowed to be static apparently
        let one_sixtieth = Duration::new(0, 1_000_000_000 / 60);
        // active split's index
        let mut current_split = 0;
        // width of the canvas
        let mut window_width: u32;
        // color of text
        let mut color = Color::WHITE;
        // used to determine if timer font map should be rerendered
        let mut old_color: Color;
        // diff between max on screen and current, used when resizing window
        let mut diff: usize;
        // number of splits
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
        // set when comparison has changed and textures need to be rerendered
        let mut comp_changed = false;
        self.canvas.present();

        // main loop
        'running: loop {
            // start measuring the time this loop pass took
            frame_time = Instant::now();
            // remove stuff from the backbuffer and fill the space with black
            self.canvas.set_draw_color(bg_color);
            self.canvas.clear();

            #[cfg(feature = "bg")]
            if has_bg {
                self.canvas
                    .copy(&bg_tex, None, bg_rect)
                    .unwrap_or_else(|err| {
                        error!("bg copy failed: {}", err);
                    });
            }
            // if the timer is doing an offset, make sure it should still be negative
            // if it shouldnt, convert to running state
            if let TimerState::OffsetCountdown { amt } = self.state {
                if amt <= total_time.elapsed().as_millis() {
                    elapsed = self.timer.elapsed().as_millis();
                    self.state = TimerState::Running { timestamp: elapsed };
                    split_time_ticks = elapsed;
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
                        if bottom_split_index < len - 1 {
                            bottom_split_index += 1;
                            top_split_index += 1;
                        }
                    }

                    // if scroll up and there are enough splits in the list, scroll splits up
                    Event::MouseWheel { y: 1, .. } => {
                        if top_split_index != 0 {
                            bottom_split_index -= 1;
                            top_split_index -= 1;
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
                        // reset stuff specific to the active run and return splits to the top of the list
                        active_run_times = vec![];
                        top_split_index = 0;
                        if max_splits != 0 {
                            bottom_split_index = max_splits - 1;
                        } else {
                            bottom_split_index = 0;
                        }
                        before_pause = 0;
                        before_pause_split = 0;
                        current_split = 0;
                        color = ahead;
                        // if there is an offset, reset the timer to that, if not, reset timer to 0
                        match offset {
                            Some(x) => {
                                self.state = TimerState::NotRunning {
                                    time_str: format!("-{}", timing::ms_to_readable(x, false)),
                                };
                            }
                            None => {
                                self.state = TimerState::NotRunning {
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
                        // calculate the height taken by the splits and the total new height of the window
                        let height = self.canvas.viewport().height();
                        let rows_height = ((bottom_split_index - top_split_index) as u32
                            * (splits_height + 2))
                            + splits_height;
                        // if there are too many splits, calculate how many and set flag to make a new list to display
                        // otherwise if there are too few and there are enough to display more, set recreate flag
                        if height - timer_height < rows_height {
                            diff =
                                ((rows_height - (height - timer_height)) / splits_height) as usize;
                            if max_splits > diff {
                                max_splits -= diff;
                            } else {
                                max_splits = 0;
                            }
                            if current_split > bottom_split_index - diff {
                                top_split_index += diff;
                                bottom_split_index = current_split;
                            } else if bottom_split_index > diff {
                                bottom_split_index -= diff;
                            } else {
                                bottom_split_index = 0;
                            }
                        } else if rows_height < height - timer_height {
                            diff =
                                (((height - timer_height) - rows_height) / splits_height) as usize;
                            if current_split == bottom_split_index
                                && current_split != len - 1
                                && top_split_index >= diff
                            {
                                top_split_index -= diff;
                                max_splits += diff;
                            } else if bottom_split_index + diff > len - 1 || max_splits + diff > len
                            {
                                bottom_split_index = len - 1;
                                max_splits = len;
                            } else {
                                max_splits += diff;
                                bottom_split_index = max_splits - 1;
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
                        TimerState::NotRunning { .. } => {
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
                                // create the difference time shown after a split
                                let sum = timing::split_time_sum(&active_run_times)[current_split];
                                let diff = sum as i128 - summed_times[current_split] as i128;
                                time_str = timing::diff_text(diff);
                                // set diff color to gold and replace split gold
                                if active_run_times[current_split] < splits[current_split].gold() {
                                    save = true;
                                    color = gold;
                                    self.run.set_gold_time(
                                        current_split,
                                        active_run_times[current_split],
                                    );
                                    splits[current_split].set_gold(active_run_times[current_split]);
                                }
                                text_surface =
                                    font.render(&time_str).blended(color).unwrap_or_else(|err| {
                                        error!("diff sur failed: {}", err);
                                    });
                                texture = creator
                                    .create_texture_from_surface(&text_surface)
                                    .unwrap_or_else(|err| {
                                        error!("diff tex failed: {}", err);
                                    });
                                splits[current_split].set_diff(diff, Some(texture));
                                time_str = timing::split_time_text((elapsed - t) + before_pause);
                                text_surface = font
                                    .render(&time_str)
                                    .blended(Color::WHITE)
                                    .unwrap_or_else(|err| {
                                        error!("cur sur failed: {}", err);
                                    });
                                texture = creator
                                    .create_texture_from_surface(&text_surface)
                                    .unwrap_or_else(|err| {
                                        error!("cur tex failed: {}", err);
                                    });
                                splits[current_split].set_cur(Some(texture));
                                // if there are still splits left, continue the run and advance the current split
                                if current_split < len - 1 {
                                    current_split += 1;
                                    // if the next split is offscreen set recreate_on_screen flag to change the current split slice
                                    if current_split > bottom_split_index
                                        && bottom_split_index + 1 < len
                                    {
                                        bottom_split_index += 1;
                                        top_split_index += 1;
                                    }
                                // otherwise end the run
                                } else {
                                    // set the state of the timer to finished, round string to 30fps
                                    self.state = TimerState::NotRunning {
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
                                                .unwrap_or_else(|err| {
                                                    error!("end comp sur failed: {}", err);
                                                });
                                            texture = creator
                                                .create_texture_from_surface(text_surface)
                                                .unwrap_or_else(|err| {
                                                    error!("end comp tex failed: {}", err);
                                                });
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
                            // finish the run if there are no splits
                            } else {
                                elapsed = self.timer.elapsed().as_millis();
                                self.state = TimerState::NotRunning {
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
                    // f1 key to open a new split file
                    Event::KeyDown {
                        keycode: Some(Keycode::F1),
                        ..
                    } => {
                        // only allow opening a new file if the timer is not running
                        if let TimerState::NotRunning { .. } = self.state {
                            // save the previous run if it was updated
                            if save && save_check() {
                                self.run.save_msf(&path);
                            }
                            // open a file dialog to get a new split file + run
                            // if the user cancelled, do nothing
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
                                    self.state = TimerState::NotRunning {
                                        time_str: format!("-{}", timing::ms_to_readable(x, false)),
                                    };
                                }
                                _ => {}
                            }
                            // recreate split names, times, textures, etc
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
                                    .unwrap_or_else(|err| {
                                        error!("f1 text sur failed: {}", err);
                                    });
                                let texture = creator
                                    .create_texture_from_surface(&text_surface)
                                    .unwrap_or_else(|err| {
                                        error!("f1 text tex failed: {}", err);
                                    });
                                let comp = font
                                    .render(&split_times_raw[index])
                                    .blended(Color::WHITE)
                                    .unwrap_or_else(|err| {
                                        error!("f1 comp sur failed: {}", err);
                                    });
                                let comp_texture = creator
                                    .create_texture_from_surface(&comp)
                                    .unwrap_or_else(|err| {
                                        error!("f1 comp tex failed: {}", err);
                                    });
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
                            // reset max splits, length of splits, and split indices to reflect new run
                            if len == 0 {
                                max_splits = ((self.canvas.viewport().height() - timer_height)
                                    / splits_height)
                                    as usize;
                            }
                            len = splits.len();
                            if max_splits > len {
                                max_splits = len;
                            }
                            top_split_index = 0;
                            if max_splits != 0 {
                                bottom_split_index = max_splits - 1;
                            } else {
                                bottom_split_index = 0;
                            }
                        }
                    }
                    _ => {}
                }
            }
            // rebuild comparisons if the comparison was swapped
            if comp_changed {
                comp_changed = false;
                index = 0;
                if let Comparison::None = self.comparison {
                    // set comp textures to just "-" if there is no comparison
                    while index < len {
                        let surface =
                            font.render("-  ")
                                .blended(Color::WHITE)
                                .unwrap_or_else(|err| {
                                    error!("comp - sur failed: {}", err);
                                });
                        let tex = creator
                            .create_texture_from_surface(&surface)
                            .unwrap_or_else(|err| {
                                error!("comp - tex failed: {}", err);
                            });
                        splits[index].set_comp_tex(tex);
                        index += 1;
                    }
                } else {
                    let split_times = match self.comparison {
                        Comparison::PersonalBest => self.run.get_times().to_vec(),
                        Comparison::Golds => self.run.get_golds().to_vec(),
                        _ => unreachable!(),
                    };
                    // rerender comparisons to either personal best or golds
                    let split_times_raw: Vec<String> = timing::split_time_sum(&split_times)
                        .iter()
                        .map(|val| timing::split_time_text(*val))
                        .collect();
                    while index < len {
                        let surface = font
                            .render(&split_times_raw[index])
                            .blended(Color::WHITE)
                            .unwrap_or_else(|err| {
                                error!("comp swap sur failed: {}", err);
                            });
                        let tex = creator
                            .create_texture_from_surface(&surface)
                            .unwrap_or_else(|err| {
                                error!("comp swap tex failed: {}", err);
                            });
                        splits[index].set_comp_tex(tex);
                        index += 1;
                    }
                }
            }
            // reset window width for placing text
            window_width = self.canvas.viewport().width();

            // make some changes to stuff before updating screen based on what happened in past loop
            // but only if the timer is running
            old_color = color;
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
                        allowed = (match self.comparison {
                            Comparison::PersonalBest => splits[current_split].time(),
                            Comparison::Golds => splits[current_split].gold(),
                            _ => unreachable!(),
                        }) as i128
                            - splits[current_split - 1].diff();
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
            // if the color has changed due to above calculations, recreate the font map in the new color
            if old_color != color {
                let map = timer_font
                    .render("- 0 1 2 3 4 5 6 7 8 9 : .")
                    .blended(color)
                    .unwrap_or_else(|err| {
                        error!("map color sur failed: {}", err);
                    });
                map_tex = creator
                    .create_texture_from_surface(&map)
                    .unwrap_or_else(|err| {
                        error!("map color tex failed: {}", err);
                    });
            }
            // copy the name, diff, and time textures to the canvas
            // and highlight the split relative to the top of the list marked by cur
            // function places the rows and ensures that they don't go offscreen
            if max_splits == 0 {
                render::render_rows(&[], &mut self.canvas, window_width, splits_height, cur);
            } else {
                render::render_rows(
                    &splits[top_split_index..=bottom_split_index],
                    &mut self.canvas,
                    window_width,
                    splits_height,
                    cur,
                );
            }
            // update the time based on the current timer state
            time_str = self.update_time(before_pause, total_time);
            // copy the time texture to the canvas, place individual characters from map
            render::render_time(time_str, &map_tex, &coords, font_y, &mut self.canvas);
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
        // if splits were updated, prompt user to save the split file
        if save && save_check() {
            self.run.save_msf(&path);
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
            TimerState::NotRunning { time_str: string }
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
