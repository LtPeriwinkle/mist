//Functions related to rendering information to the SDL window

use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, Texture, TextureQuery};
use sdl2::video::Window;

//draws split names that have been made into textures previously
pub fn render_rows(on_screen: &Vec<Texture>, canvas: &mut Canvas<Window>, window_width: u32) {
    let mut y = 0;
    for item in on_screen {
        let TextureQuery { width, height, .. } = item.query();
        let row = Rect::new(0, y, width, height);
        canvas.copy(&item, None, Some(row)).unwrap();
        canvas.set_draw_color(Color::GRAY);
        canvas
            .draw_line(
                Point::new(0, y + height as i32 + 3),
                Point::new(window_width as i32, y + height as i32 + 3),
            )
            .unwrap();
        y += height as i32 + 5;
        canvas.set_draw_color(Color::BLACK);
    }
}

pub fn render_time(texture: &Texture, canvas: &mut Canvas<Window>) {
    let vp = canvas.viewport();
    let h = vp.height();
    let w = vp.width();
    let TextureQuery { width, height, .. } = texture.query();
    let rect = Rect::new((w - width) as i32, (h - height) as i32, width, height);
    canvas.copy(&texture, None, Some(rect)).unwrap();
}
