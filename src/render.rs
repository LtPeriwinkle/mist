// Functions for putting stuff into the correct places on the sdl buffer
use crate::panels::RenderPanel;
use crate::splits::Split;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, Texture, TextureQuery};
use sdl2::video::Window;
use std::convert::TryInto;

// Puts split name textures and their associated times into the SDL backbuffer
// handles placing all the textures around the other ones and highlighting the active split based on the
// index passed to it
pub fn render_rows(
    on_screen: &[Split],
    canvas: &mut Canvas<Window>,
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
                let tinfo = item.comp_texture().query();
                row = Rect::new(
                    (window_width - tinfo.width) as i32,
                    num_y,
                    tinfo.width,
                    tinfo.height,
                );
                canvas.copy(&item.comp_texture(), None, Some(row))?;
                tinfo
            }
        };
        match item.diff_texture() {
            None => {}
            Some(x) => {
                let TextureQuery {
                    width: dw,
                    height: dh,
                    ..
                } = x.query();
                row = Rect::new(((window_width - texinfo.width - 25) - dw) as i32, y, dw, dh);
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
    canvas: &mut Canvas<Window>,
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

pub fn render_panels(panels: &[RenderPanel], canvas: &mut Canvas<Window>) -> Result<(), String> {
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

pub fn render_white_text<'a, T: ToString>(text: T, font: &sdl2::ttf::Font, creator: &'a sdl2::render::TextureCreator<sdl2::video::WindowContext>) -> Result<Texture<'a>, String> {
    let sur = font.render(&text.to_string()).blended(Color::WHITE).map_err(|_| sdl2::get_error())?;
    creator.create_texture_from_surface(sur).map_err(|_| sdl2::get_error())
}
