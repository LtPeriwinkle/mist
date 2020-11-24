//Functions related to rendering information to the SDL window

use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, Texture, TextureQuery};
use sdl2::video::Window;

// Puts split name textures and their associated times into the SDL backbuffer
pub fn render_rows(
    on_screen: &Vec<&Texture>,
    times: &Vec<&Texture>,
    canvas: &mut Canvas<Window>,
    window_width: u32,
) {
    let mut y = 0;
    let mut row: Rect;
    for item in on_screen {
        let TextureQuery { width, height, .. } = item.query();
        row = Rect::new(0, y, width, height);
        canvas
            .copy(&item, None, Some(row))
            .expect("split texture copy failed");
        canvas.set_draw_color(Color::GRAY);
        canvas
            .draw_line(
                Point::new(0, y + height as i32 + 3),
                Point::new(window_width as i32, y + height as i32 + 3),
            )
            .expect("line draw failed");
        y += height as i32 + 5;
        canvas.set_draw_color(Color::BLACK);
    }
    y = 0;
    let vpw = canvas.viewport().width();
    for item in times {
        let TextureQuery { width, height, .. } = item.query();
        row = Rect::new((vpw - width) as i32, y, width, height);
        canvas
            .copy(&item, None, Some(row))
            .expect("split time texture copy failed");
        canvas.set_draw_color(Color::GRAY);
        y += height as i32 + 5;
        canvas.set_draw_color(Color::BLACK);
    }
}

// Puts the big display timer at the bottom into the SDL backbuffer
pub fn render_time(texture: &Texture, canvas: &mut Canvas<Window>) {
    let vp = canvas.viewport();
    let h = vp.height();
    let w = vp.width();
    let TextureQuery { width, height, .. } = texture.query();
    if w > width {
    	let rect = Rect::new((w - width) as i32, (h - height) as i32, width, height);
    	canvas
            .copy(&texture, None, Some(rect))
            .expect("time texture copy failed");
    }
}
