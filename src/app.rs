use std::cell::RefCell;
use std::fs::File;
use std::io::BufReader;
use std::rc::Rc;
use std::thread;
use std::time::{Duration, Instant};

use sdl2::event::{Event, WindowEvent};
use sdl2::get_error;
#[cfg(feature = "icon")]
use sdl2::image::LoadSurface;
use sdl2::keyboard::Keycode;
#[cfg(feature = "icon")]
use sdl2::surface::Surface;

use mist_core::{
    config::Config,
    dialogs,
    parse::MsfParser,
    timer::{
        state::{RunState, RunUpdate, StateChangeRequest},
        Run,
    },
};

use crate::keybinds::Keybinds;
use crate::render::RenderState;

pub struct App<'a, 'b> {
    _context: sdl2::Sdl,
    run: Rc<RefCell<Run>>,
    ren_state: RenderState<'a, 'b>,
    run_state: RunState,
    config: Config,
    ev_pump: sdl2::EventPump,
    msf: MsfParser,
}
static ONE_SIXTIETH: Duration = Duration::new(0, 1_000_000_000 / 60);

impl<'a, 'b> App<'a, 'b> {
    pub fn init(context: sdl2::Sdl) -> Result<Self, String> {
        let video = context.video()?;
        let mut window = video
            .window("mist", 300, 500)
            .position_centered()
            .resizable()
            .build()
            .map_err(|_| get_error())?;
        #[cfg(feature = "icon")]
        {
            let icon = Surface::from_file("assets/MIST.png")?;
            window.set_icon(icon);
        }

        let mut canvas = window.into_canvas().build().map_err(|_| get_error())?;
        let ev_pump = context.event_pump()?;
        let mut config = Config::open()?;
        let msf = MsfParser::new();
        let run = Rc::new(RefCell::new(if let Some(x) = config.file() {
            let f = File::open(x).map_err(|e| e.to_string())?;
            let reader = BufReader::new(f);
            msf.parse(reader)?
        } else {
            match dialogs::open_run() {
                Ok(ret) => {
                    if let Some((r, path)) = ret {
                        config.set_file(&path);
                        r
                    } else {
                        Run::empty()
                    }
                }
                Err(e) => return Err(e.to_string()),
            }
        }));

        canvas
            .window_mut()
            .set_title(&format!(
                "mist: {} ({})",
                run.borrow().game_title(),
                run.borrow().category(),
            ))
            .map_err(|_| get_error())?;
        let app = App {
            _context: context,
            ren_state: RenderState::new(Rc::clone(&run), canvas, &config)?,
            run_state: RunState::new(Rc::clone(&run)),
            config,
            ev_pump,
            msf,
            run,
        };

        Ok(app)
    }

    pub fn run(mut self) -> Result<(), String> {
        let no_file: bool;
        let mut path = if let Some(p) = self.config.file() {
            no_file = false;
            p.clone()
        } else {
            no_file = true;
            "".to_owned()
        };

        // framerate cap timer
        let mut frame_time: Instant;
        let mut binds = Keybinds::from_raw(self.config.binds())?;
        let mut state_change_queue = vec![];
        let mut update: RunUpdate;

        // main loop
        'running: loop {
            frame_time = Instant::now();
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

                    Event::MouseWheel { y, .. } => {
                        self.ren_state.scroll(y);
                    }

                    Event::KeyDown {
                        keycode: Some(k),
                        repeat: false,
                        ..
                    } => {
                        if k == binds.start_split {
                            state_change_queue.push(StateChangeRequest::Split);
                        } else if k == binds.pause {
                            state_change_queue.push(StateChangeRequest::Pause);
                        } else if k == binds.reset {
                            state_change_queue.push(StateChangeRequest::Reset);
                        } else if k == binds.prev_comp {
                            state_change_queue.push(StateChangeRequest::Comparison(false));
                        } else if k == binds.next_comp {
                            state_change_queue.push(StateChangeRequest::Comparison(true));
                        } else if k == binds.un_split {
                            state_change_queue.push(StateChangeRequest::Unsplit);
                        } else if k == binds.load_splits {
                            // only allow opening a new file if the timer is not running
                            if !self.run_state.is_running() {
                                // save the previous run if it was updated
                                if (self.run_state.needs_save() || no_file) && dialogs::save_check()
                                {
                                    if path.is_empty() {
                                        let p = dialogs::get_save_as();
                                        if let Some(s) = p {
                                            path = s;
                                            let mut f =
                                                File::create(&path).map_err(|e| e.to_string())?;
                                            self.msf.write(&self.run.borrow(), &mut f)?;
                                        }
                                    } else {
                                        let mut f =
                                            File::create(&path).map_err(|e| e.to_string())?;
                                        self.msf.write(&self.run.borrow(), &mut f)?;
                                    }
                                }
                                // open a file dialog to get a new split file + run
                                // if the user cancelled, do nothing
                                match dialogs::open_run() {
                                    Ok(s) => {
                                        if let Some((run, p)) = s {
                                            self.run.replace(run);
                                            self.config.set_file(&path);
                                            path = p;
                                        }
                                    }
                                    Err(e) => return Err(e.to_string()),
                                }
                                self.run_state = RunState::new(Rc::clone(&self.run));
                                self.ren_state.reload_run()?;
                            }
                        } else if k == binds.skip_split {
                            state_change_queue.push(StateChangeRequest::Skip);
                        } else if k == binds.load_config {
                            match dialogs::open_config() {
                                Ok(c) => {
                                    if let Some(conf) = c {
                                        self.config = conf;
                                        self.ren_state =
                                            self.ren_state.reload_config(&self.config)?;
                                        binds = Keybinds::from_raw(self.config.binds())?;
                                    }
                                }
                                Err(e) => return Err(e),
                            }
                        }
                    }

                    Event::Window {
                        win_event: WindowEvent::Resized(_, y),
                        ..
                    } => {
                        self.ren_state.win_resize(y as u32);
                    }
                    _ => {}
                }
            }
            update = self.run_state.update(&state_change_queue[..]);
            state_change_queue.clear();
            self.ren_state.update(update)?;
            self.ren_state.render()?;
            if Instant::now().duration_since(frame_time) <= ONE_SIXTIETH {
                thread::sleep(
                    // if the entire loop pass was completed in under 1/60 second, delay to keep the framerate at ~60fps
                    ONE_SIXTIETH - Instant::now().duration_since(frame_time),
                );
            }
        }
        self.config.save()?;
        // if splits were updated, prompt user to save the split file
        if (self.run_state.needs_save() || no_file) && dialogs::save_check() {
            if path.is_empty() {
                let p = dialogs::get_save_as();
                if let Some(s) = p {
                    path = s;
                    let mut f = File::create(&path).map_err(|e| e.to_string())?;
                    self.msf.write(&self.run.borrow(), &mut f)?;
                }
            } else {
                let mut f = File::create(&path).map_err(|e| e.to_string())?;
                self.msf.write(&self.run.borrow(), &mut f)?;
            }
        }
        Ok(())
    }
}
