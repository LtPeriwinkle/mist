use mist_core::config::Panel;
use sdl2::render::Texture;

pub struct RenderPanel {
    text: Texture,
    time: Texture,
    ty: Panel,
}

impl RenderPanel {
    pub fn new(text: Texture, time: Texture, ty: Panel) -> RenderPanel {
        RenderPanel { text, time, ty }
    }
    pub fn text(&self) -> &Texture {
        &self.text
    }
    pub fn time(&self) -> &Texture {
        &self.time
    }
    pub fn set_time(&mut self, new: Texture) {
        unsafe {
            sdl2::sys::SDL_DestroyTexture(self.time.raw());
        }
        self.time = new;
    }
    pub fn panel_type(&self) -> &Panel {
        &self.ty
    }
}
