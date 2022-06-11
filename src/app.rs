use std::cell::RefCell;
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
        let mut config = Config::open()?;
        let mut window = video
            .window("mist", config.win_size().0, config.win_size().1)
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
        let (run, msf) = loop {
            let path = if let Some(x) = config.file() {
                x.to_owned()
            } else {
                match dialogs::get_run_path() {
                    Some(x) => x,
                    None => "".into(),
                }
            };
            if path.is_empty() {
                break (Run::empty(), MsfParser::new(""));
            } else {
                let msf = MsfParser::new(&path);
                match msf.parse() {
                    Ok(r) => {
                        config.set_file(&path);
                        break (r, msf);
                    }
                    Err(_) => {
                        if !dialogs::try_again() {
                            break (Run::empty(), msf);
                        }
                    }
                }
            }
        };
        let run = Rc::new(RefCell::new(run));

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
        let no_file = self.config.file().is_none();

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
                                    if self.msf.no_path() {
                                        if let Some(s) = dialogs::get_save_as() {
                                            self.msf.set_filename(&s);
                                            self.msf.write(&self.run.borrow())?;
                                        }
                                    } else {
                                        self.msf.write(&self.run.borrow())?;
                                    }
                                }
                                // open a file dialog to get a new split file + run
                                // if the user cancelled, do nothing
                                loop {
                                    if let Some(x) = dialogs::get_run_path() {
                                        self.msf.set_filename(&x);
                                        match self.msf.parse() {
                                            Ok(r) => {
                                                self.run.replace(r);
                                                break;
                                            }
                                            Err(_) => {
                                                if !dialogs::try_again() {
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                                self.config.set_file(self.msf.filename());
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
        self.config.set_win_size(self.ren_state.win_size());
        self.config.save()?;
        // if splits were updated, prompt user to save the split file
        if (self.run_state.needs_save() || no_file) && dialogs::save_check() {
            if self.msf.no_path() {
                let p = dialogs::get_save_as();
                if let Some(s) = p {
                    self.msf.set_filename(s);
                    self.msf.write(&self.run.borrow())?;
                }
            } else {
                self.msf.write(&self.run.borrow())?;
            }
        }
        Ok(())
    }
}
