use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{Texture, WindowCanvas};
use sdl2::surface::Surface;
use sdl2::ttf;
use std::thread;
use std::time::{Duration, Instant};

use crate::render;
use crate::splits;
use crate::timing;

const SPLITS_ON_SCREEN: usize = 8; //used to limit number of splits displayed

// struct that does all the everything
#[allow(dead_code)]
pub struct App {
    context: sdl2::Sdl,
    ev_pump: sdl2::EventPump,
    canvas: WindowCanvas,
    ttf: sdl2::ttf::Sdl2TtfContext,
    state: TimerState,
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
        }
    }

    pub fn run(&mut self) {
        // set up some stuff that's a pain to do elsewhere
        self.canvas.clear();
        let mut bottom_split_index = SPLITS_ON_SCREEN;
        let timer_font = self
            .ttf
            .load_font("assets/segoe-ui-bold.ttf", 60)
            .expect("could not open font file");
        let font = self
            .ttf
            .load_font("assets/segoe-ui-bold.ttf", 25)
            .expect("could not open font file");
        let creator = self.canvas.texture_creator();

        // get first vec of split name textures
        let split_names = splits::get_splits();
        let split_times_raw = splits::get_split_times();
        let mut text_surface: Surface;
        let mut texture: Texture;
        let mut on_screen: Vec<&Texture> = vec![];
        let mut on_screen_times: Vec<&Texture> = vec![];
        let mut splits: Vec<Texture> = vec![];
        let mut split_times: Vec<Texture> = vec![];

        for item in split_names {
            text_surface = font.render(item).blended(Color::WHITE).unwrap();
            texture = creator.create_texture_from_surface(text_surface).unwrap();
            splits.push(texture);
        }

        for item in split_times_raw {
            text_surface = font
                .render(&timing::ms_to_readable(item, false))
                .blended(Color::WHITE)
                .unwrap();
            texture = creator.create_texture_from_surface(text_surface).unwrap();
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
        self.canvas.present();

        // main loop
        'running: loop {
            frame_time = Instant::now();
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
                            let mut index = bottom_split_index - SPLITS_ON_SCREEN;
                            while index < bottom_split_index {
                                on_screen.push(&splits[index]);
                                on_screen_times.push(&split_times[index]);
                                index += 1;
                            }
                        }
                    }
                    // if scroll up and there are enough splits in the list, scroll splits up
                    Event::MouseWheel { y: 1, .. } => {
                        if bottom_split_index != SPLITS_ON_SCREEN {
                            bottom_split_index -= 1;
                            on_screen = vec![];
                            on_screen_times = vec![];
                            let mut index = bottom_split_index - SPLITS_ON_SCREEN;
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
                        ..
                    } => {
                        if let TimerState::Paused { time, .. } = self.state {
                            total_time = Instant::now();
                            before_pause = Some(Duration::from_millis(time as u64));
                            self.state = TimerState::Running {
                                color: Color::GREEN,
                                timestamp: event_time,
                            };
                        } else if let TimerState::Running {
                            timestamp: start_running_time,
                            ..
                        } = self.state
                        {
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
                .unwrap();
            texture = creator.create_texture_from_surface(&text_surface).unwrap();
            render::render_time(&texture, &mut self.canvas);
            self.canvas.present();
            thread::sleep(
                Duration::new(0, 1_000_000_000 / 60) - Instant::now().duration_since(frame_time),
            );
        }
    }
    // updates time string based on timer state, basically leaves it the same if timer is paused
    fn update_time(&self, before_pause: Option<Duration>, total_time: Instant) -> String {
        let time: String;
        if let TimerState::Running { .. } = self.state {
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
        } else if let TimerState::Paused { time_str, .. } = &self.state {
            time = time_str.to_string();
        } else {
            time = "a".to_string(); // have to do this because compiler doesn't know that there are a finite number of states
        }
        return time;
    }
}
