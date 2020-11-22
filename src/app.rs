use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, Texture};
use sdl2::surface::Surface;
use sdl2::ttf;
use sdl2::video::Window;
use std::thread;
use std::time::{Duration, Instant};

use crate::render;
use crate::timing;

const SPLITS_ON_SCREEN: usize = 8; //used to limit number of splits displayed

// struct that does all the everything
#[allow(dead_code)]
pub struct App {
    context: sdl2::Sdl,
    ev_pump: sdl2::EventPump,
    canvas: Canvas<Window>,
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
            context: context,
            ev_pump: ev_pump,
            canvas: canvas,
            ttf: ttf,
            state: TimerState::Paused {
                time: 0,
                time_str: "0.000".to_string(),
            }, // might be a notstarted variant sometime down the line
        }
    }

    pub fn run(&mut self) {
        // set up some stuff that's a pain to do elsewhere
        self.canvas.clear();
        let mut current_index = SPLITS_ON_SCREEN;
        let timer_font = self
            .ttf
            .load_font("assets/segoe-ui-bold.ttf", 60)
            .expect("could not open font file");
        let font = self
            .ttf
            .load_font("assets/segoe-ui-bold.ttf", 30)
            .expect("could not open font file");
        let creator = self.canvas.texture_creator();

        // get first vec of split name textures
        let splits = App::get_splits();
        let mut on_screen: Vec<Texture> = vec![];
        let mut text_surface: Surface;
        let mut texture: Texture;
        for item in splits[0..current_index].iter() {
            text_surface = font.render(item).blended(Color::WHITE).unwrap();
            texture = creator.create_texture_from_surface(&text_surface).unwrap();
            on_screen.push(texture);
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
                        if current_index < splits.len() {
                            current_index += 1;
                            on_screen = vec![];
                            for item in splits[current_index - SPLITS_ON_SCREEN..current_index].iter()
                            {
                                text_surface = font.render(item).blended(Color::WHITE).unwrap();
                                texture = creator.create_texture_from_surface(&text_surface).unwrap();
                                on_screen.push(texture);
                            }
                        }
                    }
                    // if scroll up and there are enough splits in the list, scroll splits up
                    Event::MouseWheel { y: 1, .. } => {
                        if current_index != SPLITS_ON_SCREEN {
                            current_index -= 1;
                            on_screen = vec![];
                            for item in splits[current_index - SPLITS_ON_SCREEN..current_index].iter()
                            {
                                text_surface = font.render(item).blended(Color::WHITE).unwrap();
                                texture = creator.create_texture_from_surface(&text_surface).unwrap();
                                on_screen.push(texture);
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
            render::render_rows(&on_screen, &mut self.canvas, window_width);
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
    // will move to something like `parser.rs` once split files are a thing
    fn get_splits() -> Vec<&'static str> {
        vec![
            "Something",
            "else",
            "words",
            "text",
            "split 5 idk",
            "q",
            "asdf",
            "words 2",
            "no",
            "yes",
            "another one",
        ]
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
