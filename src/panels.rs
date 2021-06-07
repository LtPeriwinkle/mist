use mist_core::config::Panel;
use sdl2::render::Texture;

pub struct RenderPanel<'a> {
    text: Texture<'a>,
    time: Texture<'a>,
    ty: Panel,
}

impl<'a> RenderPanel<'a> {
    pub fn new(text: Texture<'a>, time: Texture<'a>, ty: Panel) -> RenderPanel<'a> {
        RenderPanel { text, time, ty }
    }
    pub fn text(&self) -> &Texture {
        &self.text
    }
    pub fn time(&self) -> &Texture {
        &self.time
    }
    pub fn set_time(&mut self, new: Texture<'a>) {
        self.time = new;
    }
    pub fn panel_type(&self) -> &Panel {
        &self.ty
    }
}
