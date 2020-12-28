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
                let TextureQuery { width, height, .. } = item.pb().query();
                row = Rect::new((window_width - width) as i32, y, width, height);
                canvas
                    .copy(&item.pb(), None, Some(row))
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
// aligns the texture with the right side of the window. if the window is not wide enough to show the entire thing
// it currently just won't render it
pub fn render_time(texture: &Texture, canvas: &mut Canvas<Window>) {
    let vp = canvas.viewport();
    let h = vp.height();
    let w = vp.width();
    let TextureQuery { width, height, .. } = texture.query();
    if w > width {
        let rect = Rect::new((w - width) as i32, h as i32 - height as i32, width, height);
        canvas
            .copy(&texture, None, Some(rect))
            .expect("time texture copy failed");
    }
}
