// Functions for putting stuff into the correct places on the sdl buffer
use crate::panels::RenderPanel;
use crate::splits::Split;
use mist_core::config::{Config, Panel};
use mist_core::{timing, Run};
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
use sdl2::surface::Surface;
use sdl2::ttf::{self, Font, Sdl2TtfContext};
use sdl2::video::WindowContext;
use std::cell::RefCell;
use std::convert::TryInto;
use std::rc::Rc;

pub struct RenderState<'a> {
    run: Rc<RefCell<Run>>,
    canvas: WindowCanvas,
    creator: TextureCreator<WindowContext>,
    ttf: Sdl2TtfContext,
    colors: [(u8, u8, u8); 6],
    splits: Vec<Split<'a>>,
    panels: Vec<RenderPanel<'a>>,
    map: FontMap<'a>,
    timer_font: Font<'a, 'a>,
    splits_font: Font<'a, 'a>,
    #[cfg(feature = "bg")]
    background: Background<'a>,
}

#[cfg(feature = "bg")]
enum Background<'a> {
    NoBackground,
    HasBackground { tex: Texture<'a>, rect: Rect },
}

struct FontMap<'a> {
    tex: Texture<'a>,
    coords: Vec<u32>,
}

impl<'a> RenderState<'a> {
    pub fn new(
        run: Rc<RefCell<Run>>,
        canvas: WindowCanvas,
        config: &Config,
    ) -> Result<Self, String> {
        let ttf = ttf::init().map_err(|_| get_error())?;
        let creator = canvas.texture_creator();
        let rw = RWops::from_file(config.tfont().get_path()?, "r")?;
        let timer_font = ttf.load_font_from_rwops(rw, config.fsize().0)?;
        let rw = RWops::from_file(config.sfont().get_path()?, "r")?;
        let splits_font = ttf.load_font_from_rwops(rw, config.fsize().1)?;
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
                    timing::split_time_text(sob)
                } else {
                    "-  ".into()
                };
                let time_tex = render_white_text(&time, &splits_font, &creator)?;
                let text_tex = render_white_text(&text, &splits_font, &creator)?;
                let newpanel = RenderPanel::new(text_tex, time_tex, paneltype);
                ret.push(newpanel);
            }
            ret
        };
        Ok(Self {
            run,
            canvas,
            creator,
            ttf,
            colors: config.color_list(),
            panels,
            map: FontMap::generate(&timer_font, &creator, Color::WHITE)?,
            timer_font,
            splits_font,
            background: Background::load(config, canvas.viewport(), creator)?,
        })
    }
}

impl<'a> FontMap<'a> {
    fn generate(
        font: &Font<'a, 'a>,
        creator: &'a TextureCreator<WindowContext>,
        color: Color,
    ) -> Result<Self, String> {
        let mut max = 0;
        let mut sum = 0;
        let mut coords = vec![0];
        for chr in "-0123456789:.".chars() {
            let temp = font.size_of(&chr.to_string()).map_err(|_| get_error())?.0;
            sum += temp;
            if temp > max {
                max = temp
            };
            coords.push(sum);
        }
        let sur = font
            .render("- 0 1 2 3 4 5 6 7 8 9 : .")
            .blended(color)
            .map_err(|_| get_error())?;
        Ok(Self {
            tex: creator
                .create_texture_from_surface(&sur)
                .map_err(|_| get_error())?,
            coords,
        })
    }
}

impl Background<'_> {
    fn load(
        config: &Config,
        viewport: Rect,
        creator: TextureCreator<WindowContext>,
    ) -> Result<Self, String> {
        let bg: Option<Surface> = match config.img() {
            Some(ref p) => Some(Surface::from_file(&p)?),
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

// Puts split name textures and their associated times into the SDL backbuffer
// handles placing all the textures around the other ones and highlighting the active split based on the
// index passed to it
pub fn render_rows(
    on_screen: &[Split],
    canvas: &mut WindowCanvas,
    (split_height, inline): (u32, bool),
    current: usize,
) -> Result<(), String> {
    let incr_height: i32 = (split_height * (!inline as u32 + 1)) as i32;
    let mut y = 0;
    let mut row: Rect;
    let window_width = canvas.viewport().width();
    // draw each split name on the left of the screen
    for (index, item) in on_screen.iter().enumerate() {
        let TextureQuery { width, height, .. } = item.name().query();
        // draw the blue highlight box before drawing the text for the split with index current
        if index == current {
            canvas.set_draw_color(Color::BLUE);
            canvas.fill_rect(Rect::new(0, y - 1, window_width, incr_height as u32 + 5))?;
        }
        row = Rect::new(0, y, width, height);
        canvas.copy(&item.name(), None, Some(row))?;
        let num_y = if !inline { y + split_height as i32 } else { y };
        // if the split has a texture from an active run, draw it to reflect the current time
        // otherwise draw the pb split time
        let texinfo = match item.cur() {
            Some(x) => {
                let tinfo = x.query();
                row = Rect::new(
                    (window_width - tinfo.width) as i32,
                    num_y,
                    tinfo.width,
                    tinfo.height,
                );
                canvas.copy(&x, None, Some(row))?;
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
                canvas.copy(&item.comp(), None, Some(row))?;
                tinfo
            }
        };
        match item.diff() {
            None => {}
            Some(x) => {
                let TextureQuery {
                    width: dw,
                    height: dh,
                    ..
                } = x.query();
                row = Rect::new(
                    ((window_width - texinfo.width - 25) - dw) as i32,
                    num_y,
                    dw,
                    dh,
                );
                canvas.copy(&x, None, Some(row))?;
            }
        }
        canvas.set_draw_color(Color::GRAY);
        // draw a line to separate between the rows
        y += incr_height + 3;
        canvas.draw_line(Point::new(0, y), Point::new(window_width as i32, y))?;
        y += 2;
    }
    Ok(())
}

// Puts the big display timer at the bottom into the SDL backbuffer
// cuts the individual characters out of the font map produced earlier
// scales milliseconds down to look nicer
pub fn render_time(
    atlas: &Texture,
    coords: &[(u32, u32, u32, u32)],
    (font_y, splits_height, num_panels): (u32, u32, usize),
    canvas: &mut WindowCanvas,
) -> Result<(), String> {
    let vp = canvas.viewport();
    let h = vp.height();
    let w = vp.width();
    let mut src = Rect::new(0, 0, 0, font_y);
    // multiply initial values by 8/10 so that the font is smaller
    let mut dst = Rect::new(
        0,
        (h - (font_y * 8 / 10) - (splits_height * num_panels as u32)) as i32 - 5,
        0,
        font_y * 8 / 10,
    );
    for (idx, (sx, sw, dx, dw)) in coords.iter().enumerate() {
        src.set_x((*sx).try_into().unwrap());
        src.set_width(*sw);
        dst.set_x((w - *dx).try_into().unwrap());
        dst.set_width(*dw);
        if idx == 3 {
            dst.set_y((h - font_y - (splits_height * num_panels as u32)) as i32);
            dst.set_height(font_y);
        }
        canvas.copy(atlas, Some(src), Some(dst))?;
    }
    Ok(())
}

pub fn get_coords(time_str: String, coords: &[u32]) -> Vec<(u32, u32, u32, u32)> {
    let mut coord_idx;
    let mut ret: Vec<(u32, u32, u32, u32)> = vec![];
    let mut x = 0;
    let space = coords[14] - coords[13];
    for (idx, chr) in time_str.chars().rev().enumerate() {
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
        let width = coords[coord_idx + 1] - coords[coord_idx];
        x += if chr == ':' || chr == '.' {
            width
        } else if idx < 4 {
            coords[15] * 8 / 10
        } else {
            coords[15]
        };
        let tup = (
            coords[coord_idx] + (coord_idx as u32 * space),
            width,
            x,
            if idx < 4 { width * 8 / 10 } else { width },
        );
        ret.push(tup);
    }
    ret
}

pub fn render_panels(panels: &[RenderPanel], canvas: &mut WindowCanvas) -> Result<(), String> {
    let mut num = 1;
    for panel in panels {
        let TextureQuery { width, height, .. } = panel.text().query();
        canvas.copy(
            panel.text(),
            None,
            Some(Rect::new(
                0,
                (canvas.viewport().height() - (num * height)) as i32,
                width,
                height,
            )),
        )?;
        let TextureQuery { width, height, .. } = panel.time().query();
        canvas.copy(
            panel.time(),
            None,
            Some(Rect::new(
                (canvas.viewport().width() - width) as i32,
                (canvas.viewport().height() - (num * height)) as i32,
                width,
                height,
            )),
        )?;
        num += 1;
    }
    Ok(())
}

pub fn render_white_text<'a, T: ToString>(
    text: T,
    font: &sdl2::ttf::Font,
    creator: &'a sdl2::render::TextureCreator<sdl2::video::WindowContext>,
) -> Result<Texture<'a>, String> {
    let sur = font
        .render(&text.to_string())
        .blended(Color::WHITE)
        .map_err(|_| sdl2::get_error())?;
    creator
        .create_texture_from_surface(sur)
        .map_err(|_| sdl2::get_error())
}
