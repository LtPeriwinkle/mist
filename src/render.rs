// Functions for putting stuff into the correct places on the sdl buffer
use crate::panels::RenderPanel;
use crate::splits::Split;
use mist_core::config::{Config, Panel};
use mist_core::timer::state::{RunUpdate, SplitStatus, StateChange};
use mist_core::timer::{format, Comparison, Run};
use sdl2::get_error;
#[cfg(feature = "bg")]
use sdl2::gfx::rotozoom::RotozoomSurface;
#[cfg(feature = "bg")]
use sdl2::image::LoadSurface;
use sdl2::pixels::Color;
#[cfg(feature = "bg")]
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Texture, TextureCreator, TextureQuery, WindowCanvas};
use sdl2::rwops::RWops;
#[cfg(feature = "bg")]
use sdl2::surface::Surface;
use sdl2::ttf::{self, Font, Sdl2TtfContext};
use sdl2::video::WindowContext;
use std::cell::RefCell;
use std::convert::TryInto;
use std::rc::Rc;

const ALL_CHARS: &str =
    "AaBbCcDdEeFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz`1234567890[]~!@#$%^&*(){}',./=\\-;\"<>?+|_:";
const TIMER_CHARS: &str = "1234567890:.-";

pub struct RenderState<'a, 'b> {
    run: Rc<RefCell<Run>>,
    canvas: WindowCanvas,
    creator: TextureCreator<WindowContext>,
    colors: [(u8, u8, u8); 6],
    splits: Vec<Split>,
    panels: Vec<RenderPanel>,
    map: FontMap,
    time_str: String,
    time_rounding: Option<u128>,
    is_rounding: bool,
    timer_font: Font<'b, 'a>,
    timer_height: u32,
    splits_font: Font<'b, 'a>,
    splits_height: u32,
    top_index: usize,
    bottom_index: usize,
    highlighted: usize,
    current: usize,
    max_splits: usize,
    inline: bool,
    status: SplitStatus,
    #[cfg(feature = "bg")]
    background: Background,
}

#[cfg(feature = "bg")]
enum Background {
    NoBackground,
    HasBackground { tex: Texture, rect: Rect },
}

struct FontMap {
    tex: Texture,
    coords: Vec<u32>,
}

// wish i did not have to do this
lazy_static::lazy_static! {
    static ref TTF: Sdl2TtfContext = ttf::init().unwrap();
}

impl<'a, 'b> RenderState<'a, 'b> {
    pub fn new(
        run: Rc<RefCell<Run>>,
        mut canvas: WindowCanvas,
        config: &Config,
    ) -> Result<Self, String> {
        canvas.clear();
        let creator = canvas.texture_creator();
        let rw = RWops::from_file(config.tfont().get_path()?, "r")?;
        let timer_font = TTF.load_font_from_rwops(rw, config.fsize().0)?;
        let rw = RWops::from_file(config.sfont().get_path()?, "r")?;
        let splits_font = TTF.load_font_from_rwops(rw, config.fsize().1)?;
        let panels = {
            let mut ret = vec![];
            for panel in config.panels() {
                let (text, paneltype) = match panel {
                    Panel::Pace { golds } => {
                        if *golds {
                            ("Pace (best)", Panel::Pace { golds: true })
                        } else {
                            ("Pace (pb)", Panel::Pace { golds: false })
                        }
                    }
                    Panel::SumOfBest => ("Sum of Best", Panel::SumOfBest),
                    Panel::CurrentSplitDiff { golds } => {
                        if *golds {
                            ("Split (best)", Panel::CurrentSplitDiff { golds: true })
                        } else {
                            ("Split (pb)", Panel::CurrentSplitDiff { golds: false })
                        }
                    }
                };
                let time = if let Panel::SumOfBest = panel {
                    let sob = run.borrow().gold_times().iter().sum::<u128>();
                    format::split_time_text(sob)
                } else {
                    "-  ".into()
                };
                let time_tex = render_text(&time, &splits_font, &creator, Color::WHITE)?;
                let text_tex = render_text(&text, &splits_font, &creator, Color::WHITE)?;
                let newpanel = RenderPanel::new(text_tex, time_tex, paneltype);
                ret.push(newpanel);
            }
            ret
        };
        let string_times: Vec<String> = format::split_time_sum(run.borrow().pb_times())
            .iter()
            .map(|&t| {
                if t == 0 {
                    "-  ".into()
                } else {
                    format::split_time_text(t)
                }
            })
            .collect();
        let splits: Vec<Split> = run
            .borrow()
            .splits()
            .iter()
            .enumerate()
            .map(|(idx, name)| {
                Split::new(
                    render_text(name, &splits_font, &creator, Color::WHITE).unwrap(),
                    render_text(&string_times[idx], &splits_font, &creator, Color::WHITE).unwrap(),
                    None,
                    None,
                )
            })
            .collect();
        let splits_height = splits_font.size_of(ALL_CHARS).map_err(|_| get_error())?.1;
        let timer_height = timer_font.size_of(TIMER_CHARS).map_err(|_| get_error())?.1;
        let bottom_index: usize;
        let max_splits: usize;
        let max_initial_splits = ((canvas.viewport().height() - timer_height)
            / ((splits_height * (1 + !config.layout().inline_splits as u32)) + 5))
            as usize;
        if splits.is_empty() {
            max_splits = 0;
            bottom_index = 0;
        } else if max_initial_splits > splits.len() {
            bottom_index = splits.len() - 1;
            max_splits = splits.len();
        } else {
            max_splits = max_initial_splits;
            bottom_index = max_initial_splits - 1;
        }
        let time_str = if let Some(n) = run.borrow().offset() {
            format!("-{}", format::ms_to_readable(n, None))
        } else {
            "0.000".into()
        };
        canvas
            .window_mut()
            .set_minimum_size(0, timer_height + 20 + (splits_height * panels.len() as u32))
            .map_err(|_| get_error())?;
        canvas
            .window_mut()
            .set_size(300, 500 + (splits_height * panels.len() as u32))
            .map_err(|_| get_error())?;
        canvas
            .window_mut()
            .set_title(&format!(
                "mist: {} ({})",
                run.borrow().game_title(),
                run.borrow().category(),
            ))
            .map_err(|_| get_error())?;
        canvas.present();
        Ok(Self {
            run,
            colors: config.color_list(),
            splits,
            panels,
            map: FontMap::generate(&timer_font, &creator, Color::WHITE)?,
            time_str,
            time_rounding: config.rounding(),
            is_rounding: true,
            timer_font,
            timer_height,
            splits_font,
            splits_height,
            top_index: 0,
            bottom_index,
            highlighted: usize::MAX,
            current: 0,
            max_splits,
            inline: config.layout().inline_splits,
            status: SplitStatus::None,
            #[cfg(feature = "bg")]
            background: Background::load(config, canvas.viewport(), &creator)?,
            canvas,
            creator,
        })
    }

    pub fn update(&mut self, update: RunUpdate) -> Result<(), String> {
        if update.status != self.status {
            self.status = update.status;
            let color = match self.status {
                SplitStatus::None => (255, 255, 255),
                SplitStatus::Ahead => self.colors[0],
                SplitStatus::Behind => self.colors[1],
                SplitStatus::Gaining => self.colors[2],
                SplitStatus::Losing => self.colors[3],
                SplitStatus::Gold => self.colors[4],
            };
            self.map = FontMap::generate(&self.timer_font, &self.creator, color.into()).unwrap();
        }
        if self.status != SplitStatus::None {
            for panel in &mut self.panels {
                match panel.panel_type() {
                    Panel::Pace { golds } if self.run.borrow().pb_times()[self.current] != 0 => {
                        let r = self.run.borrow();
                        let times = if *golds { r.gold_times() } else { r.pb_times() };
                        let pace = format::split_time_text(
                            times[self.current + 1..].iter().sum::<u128>() + update.time,
                        );
                        panel.set_time(render_text(
                            pace,
                            &self.splits_font,
                            &self.creator,
                            Color::WHITE,
                        )?);
                    }
                    Panel::CurrentSplitDiff { golds }
                        if self.splits.len() > 1
                            && self.run.borrow().pb_times()[self.current] != 0 =>
                    {
                        let tm = update.time;
                        let time = if !*golds {
                            if tm < self.run.borrow().pb_times()[self.current] {
                                format::diff_text(
                                    -((self.run.borrow().pb_times()[self.current] - tm) as i128),
                                )
                            } else {
                                format::diff_text(
                                    (tm - self.run.borrow().pb_times()[self.current]) as i128,
                                )
                            }
                        } else if tm < self.run.borrow().gold_times()[self.current] {
                            format::diff_text(
                                -((self.run.borrow().gold_times()[self.current] - tm) as i128),
                            )
                        } else {
                            format::diff_text(
                                (tm - self.run.borrow().gold_times()[self.current]) as i128,
                            )
                        };
                        panel.set_time(render_text(
                            time,
                            &self.splits_font,
                            &self.creator,
                            Color::WHITE,
                        )?);
                    }
                    _ => {}
                }
            }
        }
        for change in update.change {
            match change {
                StateChange::Pause => {
                    let r = self.run.borrow();
                    self.is_rounding = true;
                    self.highlighted = usize::MAX;
                    self.time_str = format::ms_to_readable(update.time, self.time_rounding);
                    self.canvas
                        .window_mut()
                        .set_title(&format!(
                            "mist: {} ({}) [{}: {}] (paused)",
                            r.game_title(),
                            r.category(),
                            self.current + 1,
                            if r.splits().is_empty() {
                                ""
                            } else {
                                &r.splits()[self.current]
                            }
                        ))
                        .map_err(|_| get_error())?;
                }
                StateChange::Finish { .. } => {
                    self.is_rounding = true;
                    self.highlighted = usize::MAX;
                    self.canvas
                        .window_mut()
                        .set_title(&format!(
                            "mist: {} ({})",
                            self.run.borrow().game_title(),
                            self.run.borrow().category(),
                        ))
                        .map_err(|_| get_error())?;
                }
                StateChange::Unpause { .. } => {
                    let r = self.run.borrow();
                    self.is_rounding = false;
                    self.canvas
                        .window_mut()
                        .set_title(&format!(
                            "mist: {} ({}) [{}: {}]",
                            r.game_title(),
                            r.category(),
                            self.current + 1,
                            if r.splits().is_empty() {
                                ""
                            } else {
                                &r.splits()[self.current]
                            }
                        ))
                        .map_err(|_| get_error())?;
                }
                StateChange::ExitSplit {
                    status, time, diff, ..
                } => {
                    let color = match status {
                        SplitStatus::None => (255, 255, 255),
                        SplitStatus::Ahead => self.colors[0],
                        SplitStatus::Behind => self.colors[1],
                        SplitStatus::Gaining => self.colors[2],
                        SplitStatus::Losing => self.colors[3],
                        SplitStatus::Gold => {
                            for panel in &mut self.panels {
                                if *panel.panel_type() == Panel::SumOfBest {
                                    panel.set_time(render_text(
                                        format::split_time_text(
                                            self.run.borrow().gold_times().iter().sum::<u128>(),
                                        ),
                                        &self.splits_font,
                                        &self.creator,
                                        Color::WHITE,
                                    )?);
                                }
                            }
                            self.colors[4]
                        }
                    };
                    let time_str = if self.run.borrow().pb_times()[self.current] == 0 {
                        "-  ".into()
                    } else {
                        format::diff_text(diff)
                    };
                    if time == 0 {
                        self.splits[self.current].set_cur(Some(render_text(
                            "-  ",
                            &self.splits_font,
                            &self.creator,
                            Color::WHITE,
                        )?));
                    } else {
                        self.splits[self.current].set_diff(Some(render_text(
                            &time_str,
                            &self.splits_font,
                            &self.creator,
                            color.into(),
                        )?));
                        let time_str = format::split_time_text(update.time);
                        self.splits[self.current].set_cur(Some(render_text(
                            &time_str,
                            &self.splits_font,
                            &self.creator,
                            Color::WHITE,
                        )?));
                    }
                }
                StateChange::EnterSplit { idx } => {
                    self.is_rounding = false;
                    // if we just unsplitted, remove the old textures
                    if idx < self.current {
                        self.splits[idx].set_cur(None);
                        self.splits[idx].set_diff(None);
                    }
                    self.current = idx;
                    {
                        let r = self.run.borrow();
                        self.canvas
                            .window_mut()
                            .set_title(&format!(
                                "mist: {} ({}) [{}: {}]",
                                r.game_title(),
                                r.category(),
                                self.current + 1,
                                if r.splits().is_empty() {
                                    ""
                                } else {
                                    &r.splits()[self.current]
                                }
                            ))
                            .map_err(|_| get_error())?;
                    }
                    if self.current > self.bottom_index {
                        self.top_index += self.current - self.bottom_index;
                        self.bottom_index = self.current;
                    } else if self.current < self.top_index {
                        self.bottom_index -= self.top_index - self.current;
                        self.top_index = self.current;
                    }
                    self.update_highlighted();
                }
                StateChange::Reset { .. } => {
                    self.current = 0;
                    self.highlighted = usize::MAX;
                    if self.max_splits == 0 {
                        self.bottom_index = 0;
                    } else {
                        self.bottom_index = self.max_splits - 1;
                    }
                    if let Some(x) = self.run.borrow().offset() {
                        self.time_str = format!("-{}", format::ms_to_readable(x, None));
                    } else {
                        self.time_str = "0.000".into();
                    }
                    for split in &mut self.splits {
                        split.set_cur(None);
                        split.set_diff(None);
                    }
                    self.is_rounding = true;
                    self.canvas
                        .window_mut()
                        .set_title(&format!(
                            "mist: {} ({})",
                            self.run.borrow().game_title(),
                            self.run.borrow().category(),
                        ))
                        .map_err(|_| get_error())?;
                }
                StateChange::ComparisonChanged { comp } => match comp {
                    Comparison::None => {
                        for split in &mut self.splits {
                            split.set_comp(render_text(
                                "-  ",
                                &self.splits_font,
                                &self.creator,
                                Color::WHITE,
                            )?);
                        }
                    }
                    Comparison::Average => {
                        let mut i = 0;
                        let (attempts, mut times) = {
                            let mut att = vec![];
                            let mut tm = vec![];
                            for sum in self.run.borrow().sum_times() {
                                att.push(sum.0);
                                tm.push(sum.1);
                            }
                            (att, tm)
                        };
                        while i < attempts.len() {
                            times[i] /= {
                                if attempts[i] == 0 {
                                    1
                                } else {
                                    attempts[i]
                                }
                            };
                            i += 1;
                        }
                        let split_times_raw: Vec<String> = format::split_time_sum(&times)
                            .iter()
                            .map(|&val| {
                                if val == 0 {
                                    "-  ".into()
                                } else {
                                    format::split_time_text(val)
                                }
                            })
                            .collect();
                        i = 0;
                        while i < self.splits.len() {
                            self.splits[i].set_comp(render_text(
                                &split_times_raw[i],
                                &self.splits_font,
                                &self.creator,
                                Color::WHITE,
                            )?);
                            i += 1;
                        }
                    }
                    c => {
                        let split_times = match c {
                            Comparison::PersonalBest => self.run.borrow().pb_times().clone(),
                            Comparison::Golds => self.run.borrow().gold_times().clone(),
                            _ => unreachable!(),
                        };
                        let split_times_raw: Vec<String> = format::split_time_sum(&split_times)
                            .iter()
                            .map(|&val| {
                                if val == 0 {
                                    "-  ".into()
                                } else {
                                    format::split_time_text(val)
                                }
                            })
                            .collect();
                        let mut i = 0;
                        while i < self.splits.len() {
                            self.splits[i].set_comp(render_text(
                                &split_times_raw[i],
                                &self.splits_font,
                                &self.creator,
                                Color::WHITE,
                            )?);
                            i += 1;
                        }
                    }
                },
                StateChange::EnterOffset => {
                    self.is_rounding = false;
                }
                _ => {}
            }
        }
        if !self.is_rounding {
            if update.offset {
                self.time_str = format!(
                    "-{}",
                    format::ms_to_readable(self.run.borrow().offset().unwrap() - update.time, None)
                );
            } else {
                self.time_str = format::ms_to_readable(update.time, None);
            }
        }
        self.update_highlighted();
        Ok(())
    }

    pub fn scroll(&mut self, y: i32) {
        if y == -1 && !self.splits.is_empty() && self.bottom_index < self.splits.len() - 1 {
            self.bottom_index += 1;
            self.top_index += 1;
        } else if y == 1 && self.top_index != 0 {
            self.bottom_index -= 1;
            self.top_index -= 1;
        }
        self.update_highlighted();
    }

    pub fn win_resize(&mut self, y: u32) {
        let row_height = self.splits_height + 5 + (!self.inline as u32 * self.splits_height);
        let all_rows_height = row_height * self.max_splits as u32;
        let bottom_height = self.timer_height + (self.splits_height * self.panels.len() as u32);
        if y - bottom_height > all_rows_height + row_height {
            let diff = (((y - bottom_height) - all_rows_height) / row_height) as usize;
            if self.max_splits + diff < self.splits.len() {
                self.max_splits += diff;
            } else {
                self.max_splits = self.splits.len();
            }
            if self.top_index > diff {
                self.top_index -= diff;
            } else if self.top_index != 0 {
                let bottom_change = diff - self.top_index;
                self.top_index = 0;
                if self.bottom_index + bottom_change < self.splits.len() - 1 {
                    self.bottom_index += bottom_change;
                } else {
                    self.bottom_index = self.splits.len() - 1;
                }
            } else if self.bottom_index + diff < self.splits.len() - 1 {
                self.bottom_index += diff;
            } else {
                self.bottom_index = self.splits.len() - 1;
            }
        } else if y - bottom_height < all_rows_height {
            let diff = ((all_rows_height - (y - bottom_height)) / row_height) as usize + 1;
            if self.max_splits > diff {
                self.max_splits -= diff;
            } else {
                self.max_splits = 0;
                self.top_index = 0;
                self.bottom_index = 0;
                self.update_highlighted();
                return;
            }
            if self.bottom_index - diff > self.top_index {
                self.bottom_index -= diff;
            } else {
                self.bottom_index = self.top_index;
            }
        }
        self.update_highlighted();
    }

    pub fn render(&mut self) -> Result<(), String> {
        self.canvas.set_draw_color(self.colors[5]);
        self.canvas.clear();

        #[cfg(feature = "bg")]
        self.render_bg()?;

        self.render_panels()?;
        self.render_rows()?;
        self.render_time()?;
        self.canvas.present();
        Ok(())
    }

    pub fn reload_run(&mut self) -> Result<(), String> {
        let string_times: Vec<String> = format::split_time_sum(self.run.borrow().pb_times())
            .iter()
            .map(|&t| {
                if t == 0 {
                    "-  ".into()
                } else {
                    format::split_time_text(t)
                }
            })
            .collect();
        self.splits = vec![];
        for (idx, name) in self.run.borrow().splits().iter().enumerate() {
            self.splits.push(Split::new(
                render_text(name, &self.splits_font, &self.creator, Color::WHITE)?,
                render_text(
                    &string_times[idx],
                    &self.splits_font,
                    &self.creator,
                    Color::WHITE,
                )?,
                None,
                None,
            ));
        }
        if let Some(x) = self.run.borrow().offset() {
            self.time_str = format!("-{}", format::ms_to_readable(x, None));
        } else {
            self.time_str = "0.000".into();
        }
        self.top_index = 0;
        self.highlighted = usize::MAX;
        self.current = 0;
        self.status = SplitStatus::None;
        let max_initial_splits = ((self.canvas.viewport().height() - self.timer_height)
            / ((self.splits_height * (1 + !self.inline as u32)) + 5))
            as usize;
        if self.splits.is_empty() {
            self.max_splits = 0;
            self.bottom_index = 0;
        } else if max_initial_splits > self.splits.len() {
            self.max_splits = self.splits.len();
            self.bottom_index = self.splits.len() - 1;
        } else {
            self.max_splits = max_initial_splits;
            self.bottom_index = max_initial_splits - 1;
        }
        Ok(())
    }

    pub fn reload_config(self, config: &Config) -> Result<Self, String> {
        Self::new(self.run, self.canvas, config)
    }

    fn update_highlighted(&mut self) {
        if !self.is_rounding && self.current >= self.top_index && self.current <= self.bottom_index
        {
            self.highlighted = self.current - self.top_index;
        } else {
            self.highlighted = usize::MAX;
        }
    }

    fn render_rows(&mut self) -> Result<(), String> {
        let on_screen = if self.max_splits > 0 {
            &self.splits[self.top_index..=self.bottom_index]
        } else {
            &[]
        };
        let incr_height: i32 = (self.splits_height * (!self.inline as u32 + 1)) as i32;
        let mut y = 0;
        let mut row: Rect;
        let window_width = self.canvas.viewport().width();
        // draw each split name on the left of the screen
        for (index, item) in on_screen.iter().enumerate() {
            let TextureQuery { width, height, .. } = item.name().query();
            // draw the blue highlight box before drawing the text for the split with index current
            if index == self.highlighted {
                self.canvas.set_draw_color(Color::BLUE);
                self.canvas
                    .fill_rect(Rect::new(0, y - 1, window_width, incr_height as u32 + 5))?;
            }
            row = Rect::new(0, y, width, height);
            self.canvas.copy(item.name(), None, Some(row))?;
            let num_y = if self.inline {
                y
            } else {
                y + self.splits_height as i32
            };
            // if the split has a texture from an active run, draw it to reflect the current time
            // otherwise draw the pb split time
            let TextureQuery {
                width: tinfo_width, ..
            } = match item.cur() {
                Some(x) => {
                    let tinfo = x.query();
                    row = Rect::new(
                        (window_width - tinfo.width) as i32,
                        num_y,
                        tinfo.width,
                        tinfo.height,
                    );
                    self.canvas.copy(x, None, Some(row))?;
                    tinfo
                }
                None => {
                    let tinfo = item.comp().query();
                    row = Rect::new(
                        (window_width - tinfo.width) as i32,
                        num_y,
                        tinfo.width,
                        tinfo.height,
                    );
                    self.canvas.copy(item.comp(), None, Some(row))?;
                    tinfo
                }
            };
            if let Some(x) = item.diff() {
                let TextureQuery {
                    width: dw,
                    height: dh,
                    ..
                } = x.query();
                row = Rect::new(
                    ((window_width - tinfo_width - 25) - dw) as i32,
                    num_y,
                    dw,
                    dh,
                );
                self.canvas.copy(x, None, Some(row))?;
            }
            self.canvas.set_draw_color(Color::GRAY);
            // draw a line to separate between the rows
            y += incr_height + 3;
            self.canvas
                .draw_line(Point::new(0, y), Point::new(window_width as i32, y))?;
            y += 2;
        }
        Ok(())
    }

    fn render_time(&mut self) -> Result<(), String> {
        let coords = self.map.gen_str_coords(&self.time_str);
        let vp = self.canvas.viewport();
        let h = vp.height();
        let w = vp.width();
        let font_y = self.timer_height;
        let mut src = Rect::new(0, 0, 0, font_y);
        // multiply initial values by 8/10 so that the font is smaller
        let mut dst = Rect::new(
            0,
            (h - (font_y * 8 / 10) - (self.splits_height * self.panels.len() as u32)) as i32 - 5,
            0,
            font_y * 8 / 10,
        );
        for (idx, (sx, sw, dx, dw)) in coords.iter().enumerate() {
            src.set_x((*sx).try_into().unwrap());
            src.set_width(*sw);
            dst.set_x((w - *dx).try_into().unwrap());
            dst.set_width(*dw);
            if idx == 3 {
                dst.set_y((h - font_y - (self.splits_height * self.panels.len() as u32)) as i32);
                dst.set_height(font_y);
            }
            self.canvas.copy(&self.map.tex, Some(src), Some(dst))?;
        }
        Ok(())
    }

    fn render_panels(&mut self) -> Result<(), String> {
        let mut num = 1;
        for panel in &self.panels {
            let TextureQuery { width, height, .. } = panel.text().query();
            self.canvas.copy(
                panel.text(),
                None,
                Some(Rect::new(
                    0,
                    (self.canvas.viewport().height() - (num * height)) as i32,
                    width,
                    height,
                )),
            )?;
            let TextureQuery { width, height, .. } = panel.time().query();
            self.canvas.copy(
                panel.time(),
                None,
                Some(Rect::new(
                    (self.canvas.viewport().width() - width) as i32,
                    (self.canvas.viewport().height() - (num * height)) as i32,
                    width,
                    height,
                )),
            )?;
            num += 1;
        }
        Ok(())
    }

    #[cfg(feature = "bg")]
    fn render_bg(&mut self) -> Result<(), String> {
        if let Background::HasBackground { ref tex, rect } = self.background {
            self.canvas.copy(tex, None, Some(rect))?;
        }
        Ok(())
    }
}

impl FontMap {
    fn generate(
        font: &Font<'_, '_>,
        creator: &TextureCreator<WindowContext>,
        color: Color,
    ) -> Result<Self, String> {
        let mut max = 0;
        let mut sum = 0;
        let mut coords = vec![0];
        for chr in "-0123456789:. ".chars() {
            let temp = font.size_of(&chr.to_string()).map_err(|_| get_error())?.0;
            sum += temp;
            if temp > max {
                max = temp;
            }
            coords.push(sum);
        }
        coords.push(max);
        let surface = font
            .render("- 0 1 2 3 4 5 6 7 8 9 : .")
            .blended(color)
            .map_err(|_| get_error())?;
        Ok(Self {
            tex: creator
                .create_texture_from_surface(&surface)
                .map_err(|_| get_error())?,
            coords,
        })
    }

    fn gen_str_coords(&self, string: &str) -> Vec<(u32, u32, u32, u32)> {
        let mut coord_idx;
        let mut ret: Vec<(u32, u32, u32, u32)> = vec![];
        let mut x = 0;
        let space = self.coords[14] - self.coords[13];
        for (idx, chr) in string.chars().rev().enumerate() {
            coord_idx = match chr {
                '-' => 0,
                '0' => 1,
                '1' => 2,
                '2' => 3,
                '3' => 4,
                '4' => 5,
                '5' => 6,
                '6' => 7,
                '7' => 8,
                '8' => 9,
                '9' => 10,
                ':' => 11,
                '.' => 12,
                _ => 0,
            };
            let width = self.coords[coord_idx + 1] - self.coords[coord_idx];
            x += if chr == ':' || chr == '.' {
                width
            } else if idx < 4 {
                self.coords[15] * 8 / 10
            } else {
                self.coords[15]
            };
            let tup = (
                self.coords[coord_idx] + (coord_idx as u32 * space),
                width,
                x,
                if idx < 4 { width * 8 / 10 } else { width },
            );
            ret.push(tup);
        }
        ret
    }
}

#[cfg(feature = "bg")]
impl Background {
    fn load(
        config: &Config,
        viewport: Rect,
        creator: &TextureCreator<WindowContext>,
    ) -> Result<Self, String> {
        let bg: Option<Surface> = match config.img() {
            Some(p) => Some(Surface::from_file(&p)?),
            None => None,
        };
        if let Some(x) = bg {
            let bg_tex: Texture;
            let width = viewport.width();
            let height = viewport.height();
            if !config.img_scaled() {
                let mut sur = Surface::new(width, height, PixelFormatEnum::RGB24)?;
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
                x.blit(Rect::new(cutoffx, cutoffy, width, height), &mut sur, None)?;
                bg_tex = creator
                    .create_texture_from_surface(&sur)
                    .map_err(|_| get_error())?;
            } else {
                let sur: Surface;
                if x.width() > x.height() && width < x.width() {
                    if width < x.width() {
                        sur = x.rotozoom(0.0, width as f64 / x.width() as f64, true)?;
                    } else {
                        sur = x.rotozoom(0.0, x.width() as f64 / width as f64, true)?;
                    }
                } else if height < x.height() {
                    sur = x.rotozoom(0.0, height as f64 / x.height() as f64, true)?;
                } else {
                    sur = x.rotozoom(0.0, x.height() as f64 / height as f64, true)?;
                }
                bg_tex = creator
                    .create_texture_from_surface(&sur)
                    .map_err(|_| get_error())?;
            }
            let sdl2::render::TextureQuery {
                width: bgw,
                height: bgh,
                ..
            } = bg_tex.query();
            Ok(Background::HasBackground {
                tex: bg_tex,
                rect: Rect::new(0, 0, bgw, bgh),
            })
        } else {
            Ok(Background::NoBackground)
        }
    }
}

fn render_text<T: ToString>(
    text: T,
    font: &sdl2::ttf::Font,
    creator: &sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    color: sdl2::pixels::Color,
) -> Result<Texture, String> {
    let sur = font
        .render(&text.to_string())
        .blended(color)
        .map_err(|_| get_error())?;
    creator
        .create_texture_from_surface(sur)
        .map_err(|_| get_error())
}
