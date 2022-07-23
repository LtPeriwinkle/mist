use crate::panels::RenderPanel;
use crate::splits::Split;
use mist_core::timer::dump::StateDump;
use mist_core::{
    config::{Colors, Config, Panel},
    timer::{
        format,
        state::{RunUpdate, SplitStatus, StateChange},
        Comparison, Run,
    },
};
use sdl2::{
    get_error,
    pixels::Color,
    rect::{Point, Rect},
    render::{Texture, TextureCreator, TextureQuery, WindowCanvas},
    rwops::RWops,
    ttf::{self, Font, Sdl2TtfContext},
    video::WindowContext,
};
#[cfg(feature = "bg")]
use sdl2::{
    gfx::rotozoom::RotozoomSurface, image::LoadSurface, pixels::PixelFormatEnum, surface::Surface,
};
use std::{cell::RefCell, convert::TryInto, rc::Rc};

const ALL_CHARS: &str =
    "AaBbCcDdEeFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz`1234567890[]~!@#$%^&*(){}',./=\\-;\"<>?+|_:";
const TIMER_CHARS: &str = "1234567890:.-";

pub struct RenderState<'a, 'b> {
    run: Rc<RefCell<Run>>,
    canvas: WindowCanvas,
    creator: TextureCreator<WindowContext>,
    colors: Colors,
    splits: Vec<Split>,
    panels: Vec<RenderPanel>,
    map: FontMap,
    time_str: String,
    time_rounding: Option<u128>,
    is_running: bool,
    rebuild: bool,
    timer_font: Font<'b, 'a>,
    timer_height: u32,
    splits_font: Font<'b, 'a>,
    splits_height: u32,
    ms_ratio: f32,
    top_index: usize,
    bottom_index: usize,
    highlighted: usize,
    current: usize,
    max_splits: usize,
    inline: bool,
    status: SplitStatus,
    comparison: Comparison,
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
        let tfont = config.tfont();
        let tf_path = tfont.get_path()?;
        let sfont = config.sfont();
        let sf_path = sfont.get_path()?;
        let rw = RWops::from_file(tf_path.0, "r")?;
        let timer_font = TTF.load_font_at_index_from_rwops(rw, tf_path.1, tfont.size())?;
        let rw = RWops::from_file(sf_path.0, "r")?;
        let splits_font = TTF.load_font_at_index_from_rwops(rw, sf_path.1, sfont.size())?;
        let panels = {
            let mut ret = vec![];
            for &panel in config.panels() {
                let (text, paneltype) = match panel {
                    p @ Panel::Pace { golds } => {
                        let text = if golds { "Pace (best)" } else { "Pace (pb)" };
                        (text, p)
                    }
                    p @ Panel::SumOfBest => ("Sum of Best", p),
                    p @ Panel::CurrentSplitDiff { golds } => {
                        let text = if golds { "Split (best)" } else { "Split (pb)" };
                        (text, p)
                    }
                };
                let time = if let Panel::SumOfBest = panel {
                    let sob = run
                        .borrow()
                        .gold_times()
                        .iter()
                        .map(|t| t.val())
                        .sum::<u128>();
                    format::split_time_text(sob)
                } else {
                    "-  ".into()
                };
                let time_tex = render_text(&time, &splits_font, &creator, config.colors().text)?;
                let text_tex = render_text(&text, &splits_font, &creator, config.colors().text)?;
                let newpanel = RenderPanel::new(text_tex, time_tex, paneltype);
                ret.push(newpanel);
            }
            ret
        };
        let times = run.borrow().pb_times().clone();
        let string_times: Vec<String> = format::split_time_sum(&run.borrow().pb_times_u128())
            .iter()
            .enumerate()
            .map(|(idx, &t)| {
                if t == 0 || !times[idx].is_time() {
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
                    render_text(name, &splits_font, &creator, config.colors().text).unwrap(),
                    render_text(
                        &string_times[idx],
                        &splits_font,
                        &creator,
                        config.colors().text,
                    )
                    .unwrap(),
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
            / ((splits_height * (1 + !config.inline_splits() as u32)) + 5))
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
        let time_str = if run.borrow().offset().is_time() {
            format!(
                "-{}",
                format::ms_to_readable(run.borrow().offset().val(), None)
            )
        } else {
            "0.000".into()
        };
        canvas
            .window_mut()
            .set_minimum_size(
                100,
                timer_height + 20 + (splits_height * panels.len() as u32),
            )
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
            colors: config.colors(),
            splits,
            panels,
            map: FontMap::generate(&timer_font, &creator, config.colors().text)?,
            time_str,
            time_rounding: config.rounding(),
            is_running: false,
            rebuild: false,
            timer_font,
            timer_height,
            splits_font,
            splits_height,
            ms_ratio: config.ms_ratio(),
            top_index: 0,
            bottom_index,
            highlighted: usize::MAX,
            current: 0,
            max_splits,
            inline: config.inline_splits(),
            status: SplitStatus::None,
            comparison: Comparison::PersonalBest,
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
                SplitStatus::None => self.colors.text,
                SplitStatus::Ahead => self.colors.ahead,
                SplitStatus::Behind => self.colors.behind,
                SplitStatus::Gaining => self.colors.gaining,
                SplitStatus::Losing => self.colors.losing,
                SplitStatus::Gold => self.colors.gold,
            };
            self.map = FontMap::generate(&self.timer_font, &self.creator, color).unwrap();
        }
        if self.status != SplitStatus::None {
            for panel in &mut self.panels {
                match *panel.panel_type() {
                    Panel::Pace { golds }
                        if self.run.borrow().pb_times()[self.current].raw() != 0 =>
                    {
                        let r = self.run.borrow();
                        let times = if golds { r.gold_times() } else { r.pb_times() };
                        let pace = format::split_time_text(
                            times[self.current + 1..]
                                .iter()
                                .map(|t| t.val())
                                .sum::<u128>()
                                + update.time,
                        );
                        panel.set_time(render_text(
                            pace,
                            &self.splits_font,
                            &self.creator,
                            self.colors.text,
                        )?);
                    }
                    Panel::CurrentSplitDiff { golds }
                        if self.splits.len() > 1
                            && self.run.borrow().pb_times()[self.current].is_time() =>
                    {
                        let compare_time: u128 = if golds {
                            self.run.borrow().gold_times()[self.current].raw()
                        } else {
                            self.run.borrow().pb_times()[self.current].raw()
                        };
                        let time = if !golds {
                            if update.split_time < compare_time {
                                format::diff_text(-((compare_time - update.split_time) as i128))
                            } else {
                                format::diff_text((update.split_time - compare_time) as i128)
                            }
                        } else if update.split_time < compare_time {
                            format::diff_text(-((compare_time - update.split_time) as i128))
                        } else {
                            format::diff_text((update.split_time - compare_time) as i128)
                        };
                        panel.set_time(render_text(
                            time,
                            &self.splits_font,
                            &self.creator,
                            self.colors.text,
                        )?);
                    }
                    _ => {}
                }
            }
        }
        if self.rebuild {
            self.rebuild = false;
            self.rebuild_comparison()?;
        }
        for change in update.change {
            match change {
                StateChange::Pause => {
                    self.is_running = false;
                    self.highlighted = usize::MAX;
                    self.time_str = format::ms_to_readable(update.time, self.time_rounding);
                }
                StateChange::Finish { .. } => {
                    self.is_running = false;
                    self.time_str = format::ms_to_readable(update.time, self.time_rounding);
                    self.highlighted = usize::MAX;
                    self.rebuild = true;
                }
                StateChange::Unpause { .. } => {
                    self.is_running = true;
                }
                StateChange::ExitSplit {
                    status, time, diff, ..
                } => {
                    if !self.run.borrow().splits().is_empty() {
                        let color = match status {
                            SplitStatus::None => self.colors.text,
                            SplitStatus::Ahead => self.colors.ahead,
                            SplitStatus::Behind => self.colors.behind,
                            SplitStatus::Gaining => self.colors.gaining,
                            SplitStatus::Losing => self.colors.losing,
                            SplitStatus::Gold => {
                                for panel in &mut self.panels {
                                    if *panel.panel_type() == Panel::SumOfBest {
                                        panel.set_time(render_text(
                                            format::split_time_text(
                                                self.run
                                                    .borrow()
                                                    .gold_times()
                                                    .iter()
                                                    .map(|t| t.val())
                                                    .sum::<u128>(),
                                            ),
                                            &self.splits_font,
                                            &self.creator,
                                            self.colors.text,
                                        )?);
                                    }
                                }
                                self.colors.gold
                            }
                        };
                        let time_str = if !self.run.borrow().pb_times()[self.current].is_time() {
                            "-  ".into()
                        } else {
                            format::diff_text(diff)
                        };
                        if time == 0 {
                            self.splits[self.current].set_cur(Some(render_text(
                                "-  ",
                                &self.splits_font,
                                &self.creator,
                                self.colors.text,
                            )?));
                        } else {
                            self.splits[self.current].set_diff(Some(render_text(
                                &time_str,
                                &self.splits_font,
                                &self.creator,
                                color,
                            )?));
                            let time_str = format::split_time_text(update.time);
                            self.splits[self.current].set_cur(Some(render_text(
                                &time_str,
                                &self.splits_font,
                                &self.creator,
                                self.colors.text,
                            )?));
                        }
                    }
                }
                StateChange::EnterSplit { idx } => {
                    self.is_running = true;
                    // if we just unsplitted, remove the old textures
                    if idx < self.current {
                        self.splits[idx].set_cur(None);
                        self.splits[idx].set_diff(None);
                    }
                    self.current = idx;
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
                    if self.run.borrow().offset().is_time() {
                        self.time_str = format!(
                            "-{}",
                            format::ms_to_readable(self.run.borrow().offset().val(), None)
                        );
                    } else {
                        self.time_str = "0.000".into();
                    }
                    for split in &mut self.splits {
                        split.set_cur(None);
                        split.set_diff(None);
                    }
                    for panel in &mut self.panels {
                        if !matches!(panel.panel_type(), Panel::SumOfBest) {
                            panel.set_time(render_text(
                                "-  ",
                                &self.splits_font,
                                &self.creator,
                                self.colors.text,
                            )?);
                        }
                    }
                    self.is_running = false;
                }
                StateChange::ComparisonChanged { comp } => {
                    self.comparison = comp;
                    self.rebuild = true;
                }
                StateChange::EnterOffset => {
                    self.is_running = true;
                }
                _ => {}
            }
        }
        if self.is_running {
            if update.offset {
                self.time_str = format!(
                    "-{}",
                    format::ms_to_readable(self.run.borrow().offset().val() - update.time, None)
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
            } else if !self.splits.is_empty() && self.bottom_index + diff < self.splits.len() - 1 {
                self.bottom_index += diff;
            } else if !self.splits.is_empty() {
                self.bottom_index = self.splits.len() - 1;
            } else {
                self.bottom_index = 0;
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
        self.canvas.set_draw_color(self.colors.background);
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
        let string_times: Vec<String> = format::split_time_sum(&self.run.borrow().pb_times_u128())
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
                render_text(name, &self.splits_font, &self.creator, self.colors.text)?,
                render_text(
                    &string_times[idx],
                    &self.splits_font,
                    &self.creator,
                    self.colors.text,
                )?,
                None,
                None,
            ));
        }
        if self.run.borrow().offset().is_time() {
            self.time_str = format!(
                "-{}",
                format::ms_to_readable(self.run.borrow().offset().val(), None)
            );
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

    pub fn win_size(&self) -> (u32, u32) {
        self.canvas.window().size()
    }

    fn update_highlighted(&mut self) {
        if self.is_running && self.current >= self.top_index && self.current <= self.bottom_index {
            self.highlighted = self.current - self.top_index;
        } else {
            self.highlighted = usize::MAX;
        }
    }

    fn rebuild_comparison(&mut self) -> Result<(), String> {
        match self.comparison {
            Comparison::None => {
                for split in &mut self.splits {
                    split.set_comp(render_text(
                        "-  ",
                        &self.splits_font,
                        &self.creator,
                        self.colors.text,
                    )?);
                }
            }
            Comparison::Average => {
                let mut i = 0;
                let (attempts, times) = {
                    let mut att = vec![];
                    let mut tm = vec![];
                    for sum in self.run.borrow().sum_times() {
                        att.push(sum.0);
                        tm.push(sum.1);
                    }
                    (att, tm)
                };
                let mut avgs = vec![];
                while i < attempts.len() {
                    avgs.push(if attempts[i] != 0 {
                        times[i] / attempts[i]
                    } else {
                        attempts[i]
                    });
                    i += 1;
                }
                let split_times_raw: Vec<String> = format::split_time_sum(&avgs)
                    .iter()
                    .enumerate()
                    .map(|(idx, &t)| {
                        if t == 0 || !times[idx].is_time() {
                            "-  ".into()
                        } else {
                            format::split_time_text(t)
                        }
                    })
                    .collect();
                i = 0;
                while i < self.splits.len() {
                    self.splits[i].set_comp(render_text(
                        &split_times_raw[i],
                        &self.splits_font,
                        &self.creator,
                        self.colors.text,
                    )?);
                    i += 1;
                }
            }
            c => {
                let split_times = match c {
                    Comparison::PersonalBest => self.run.borrow().pb_times_u128(),
                    Comparison::Golds => self.run.borrow().gold_times_u128(),
                    _ => unreachable!(),
                };
                let times = self.run.borrow().pb_times().clone();
                let split_times_raw: Vec<String> = format::split_time_sum(&split_times)
                    .iter()
                    .enumerate()
                    .map(|(idx, &t)| {
                        if t == 0 || !times[idx].is_time() {
                            "-  ".into()
                        } else {
                            format::split_time_text(t)
                        }
                    })
                    .collect();
                let mut i = 0;
                while i < self.splits.len() {
                    self.splits[i].set_comp(render_text(
                        &split_times_raw[i],
                        &self.splits_font,
                        &self.creator,
                        self.colors.text,
                    )?);
                    i += 1;
                }
            }
        }
        Ok(())
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
                self.canvas.set_draw_color(self.colors.highlight);
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
            self.canvas.set_draw_color(self.colors.line);
            // draw a line to separate between the rows
            y += incr_height + 3;
            self.canvas
                .draw_line(Point::new(0, y), Point::new(window_width as i32, y))?;
            y += 2;
        }
        Ok(())
    }

    fn render_time(&mut self) -> Result<(), String> {
        let coords = self.map.gen_str_coords(&self.time_str, self.ms_ratio);
        let vp = self.canvas.viewport();
        let h = vp.height();
        let w = vp.width();
        let mut src = Rect::new(0, 0, 0, self.timer_height);
        let starting_y = (self.timer_height as f32 * self.ms_ratio) as u32;
        let diff =
            (self.timer_height - self.timer_font.find_glyph_metrics('0').unwrap().maxy as u32) / 2;
        let mut dst = Rect::new(
            0,
            (h - (starting_y + (diff as f32 * (1.0 - self.ms_ratio)) as u32)
                - (self.splits_height * self.panels.len() as u32)) as i32,
            0,
            starting_y,
        );
        for (idx, &(sx, sw, dx, dw)) in coords.iter().enumerate() {
            src.set_x(sx.try_into().unwrap());
            src.set_width(sw);
            let wdx: i32 = if w < dx {
                let tmp: i32 = (dx - w).try_into().unwrap();
                -tmp
            } else {
                (w - dx).try_into().unwrap()
            };
            dst.set_x(wdx);
            dst.set_width(dw);
            if idx == 3 {
                dst.set_y(
                    (h - self.timer_height - (self.splits_height * self.panels.len() as u32))
                        as i32,
                );
                dst.set_height(self.timer_height);
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

    pub fn fill_dump(&self, dump: &mut StateDump) {
        dump.set_render_info(self.top_index, self.bottom_index, self.time_str.clone());
    }

    pub fn read_dump(&mut self, dump: &StateDump) -> Result<(), String> {
        self.top_index = dump.top_index;
        self.bottom_index = dump.bottom_index;
        self.time_str = dump.time_str.clone();
        self.status = dump.status;
        self.comparison = dump.comparison;
        self.current = dump.current_split;
        self.rebuild_comparison()?;
        self.rebuild_current(dump)?;
        Ok(())
    }

    fn rebuild_current(&mut self, dump: &StateDump) -> Result<(), String> {
        let times = format::split_time_sum(
            &dump
                .run_times
                .iter()
                .map_while(|v| if v.val() != 0 { Some(v.val()) } else { None })
                .collect::<Vec<_>>(),
        );
        let diff_sums =
            format::split_time_sum(&dump.run_diffs.iter().map(|v| v.raw()).collect::<Vec<_>>());
        for (i, &time) in times.iter().enumerate() {
            let time_str = format::split_time_text(time);
            self.splits[i].set_cur(Some(render_text(
                &time_str,
                &self.splits_font,
                &self.creator,
                self.colors.text,
            )?));
            let time_str = format::diff_text(diff_sums[i]);
            self.splits[i].set_diff(Some(render_text(
                &time_str,
                &self.splits_font,
                &self.creator,
                self.colors.text,
            )?));
        }
        Ok(())
    }
}

impl FontMap {
    fn generate<C: Into<Color>>(
        font: &Font<'_, '_>,
        creator: &TextureCreator<WindowContext>,
        color: C,
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

    fn gen_str_coords(&self, string: &str, ms_ratio: f32) -> Vec<(u32, u32, u32, u32)> {
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
            } else if idx < 3 {
                (self.coords[15] as f32 * ms_ratio) as u32
            } else {
                self.coords[15]
            };
            let tup = (
                self.coords[coord_idx] + (coord_idx as u32 * space),
                width,
                x,
                if idx < 3 {
                    (width as f32 * ms_ratio) as u32
                } else {
                    width
                },
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

fn render_text<T: ToString, C: Into<Color>>(
    text: T,
    font: &sdl2::ttf::Font,
    creator: &sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    color: C,
) -> Result<Texture, String> {
    let sur = font
        .render(&text.to_string())
        .blended(color)
        .map_err(|_| get_error())?;
    creator
        .create_texture_from_surface(sur)
        .map_err(|_| get_error())
}
