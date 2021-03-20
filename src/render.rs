// Functions for putting stuff into the correct places on the sdl buffer

use crate::splits::Split;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, Texture, TextureQuery};
use sdl2::video::Window;

// Puts split name textures and their associated times into the SDL backbuffer
// handles placing all the textures around the other ones and highlighting the active split based on the
// index passed to it
pub fn render_rows(
    on_screen: &[Split],
    canvas: &mut Canvas<Window>,
    window_width: u32,
    current: usize,
) {
    let mut y = 0;
    let mut row: Rect;
    let mut index = 0;
    // draw each split name on the left of the screen
    for item in on_screen {
        let TextureQuery { width, height, .. } = item.name().query();
        // draw the blue highlight box before drawing the text for the split with index current
        if index == current {
            canvas.set_draw_color(Color::BLUE);
            canvas
                .fill_rect(Rect::new(0, y - 1, window_width, height + 3))
                .unwrap();
        }
        row = Rect::new(0, y, width, height);
        canvas
            .copy(&item.name(), None, Some(row))
            .expect("split texture copy failed");
        // if the split has a texture from an active run, draw it to reflect the current time
        // otherwise draw the pb split time
        match item.cur() {
            Some(x) => {
                let TextureQuery { width, height, .. } = x.query();
                row = Rect::new((window_width - width) as i32, y, width, height);
                canvas
                    .copy(&x, None, Some(row))
                    .expect("split time texture copy failed");
                match item.diff_texture() {
                    None => {}
                    Some(x) => {
                        let TextureQuery {
                            width: dw,
                            height: dh,
                            ..
                        } = x.query();
                        row = Rect::new(((window_width - width - 25) - dw) as i32, y, dw, dh);
                        canvas
                            .copy(&x, None, Some(row))
                            .expect("split time texture copy failed");
                    }
                }
            }
            None => {
                let TextureQuery { width, height, .. } = item.comp_texture().query();
                row = Rect::new((window_width - width) as i32, y, width, height);
                canvas
                    .copy(&item.comp_texture(), None, Some(row))
                    .expect("split time texture copy failed");
                match item.diff_texture() {
                    None => {}
                    Some(x) => {
                        let TextureQuery {
                            width: dw,
                            height: dh,
                            ..
                        } = x.query();
                        row = Rect::new(((window_width - width - 25) - dw) as i32, y, dw, dh);
                        canvas
                            .copy(&x, None, Some(row))
                            .expect("split time texture copy failed");
                    }
                }
            }
        }
        canvas.set_draw_color(Color::GRAY);
        // draw a line to separate between the rows
        canvas
            .draw_line(
                Point::new(0, y + height as i32 + 3),
                Point::new(window_width as i32, y + height as i32 + 3),
            )
            .expect("line draw failed");
        y += height as i32 + 5;
        index += 1;
    }
}

// Puts the big display timer at the bottom into the SDL backbuffer
// cuts the individual characters out of the font map produced earlier
// scales milliseconds down to look nicer
pub fn render_time(
    time_str: String,
    atlas: &Texture,
    coords: &[u32],
    font_y: u32,
    canvas: &mut Canvas<Window>,
) {
    let mut x = 0;
    let vp = canvas.viewport();
    let h = vp.height();
    let w = vp.width();
    let mut src = Rect::new(0, 0, 0, font_y);
    // multiply initial values by 8/10 so that the font is smaller
    let mut dst = Rect::new(0, (h - (font_y * 8 / 10)) as i32 - 5, 0, font_y * 8 / 10);
    let mut idx: usize;
    let mut char_num = 0;
    for chr in time_str.chars().rev() {
        // get the index in the coordinate slice based on the character to render
        idx = match chr {
            '-' => 0,
            '0' => 2,
            '1' => 4,
            '2' => 6,
            '3' => 8,
            '4' => 10,
            '5' => 12,
            '6' => 14,
            '7' => 16,
            '8' => 18,
            '9' => 20,
            ':' => 22,
            '.' => 24,
            _ => 0,
        };
        let width = coords[idx + 1] - coords[idx];
        // only monospace numbers so that the typically smaller punctuation looks better
        if chr == '.' || chr == ':' {
            x += width;
        } else {
            if char_num < 4 {
                x += coords[26] * 8 / 10;
            } else {
                x += coords[26];
            }
        }
        src.set_x((coords[idx] - 2) as i32);
        src.set_width(width);
        dst.set_x((w - x) as i32);
        if char_num < 4 {
            dst.set_width(width * 8 / 10);
        } else {
            dst.set_width(width);
            dst.set_y((h - font_y) as i32);
            dst.set_height(font_y);
        }
        canvas.copy(atlas, Some(src), Some(dst)).unwrap();
        char_num += 1;
    }
}
