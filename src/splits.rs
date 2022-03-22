use sdl2::render::Texture;

pub struct Split {
    diff: Option<Texture>,
    name: Texture,
    comp: Texture,
    current: Option<Texture>,
}

impl Split {
    pub fn new(
        name: Texture,
        comp: Texture,
        diff: Option<Texture>,
        current: Option<Texture>,
    ) -> Self {
        Self {
            diff,
            name,
            comp,
            current,
        }
    }
    pub fn name(&self) -> &Texture {
        &self.name
    }
    pub fn comp(&self) -> &Texture {
        &self.comp
    }
    pub fn cur(&self) -> &Option<Texture> {
        &self.current
    }
    pub fn diff(&self) -> &Option<Texture> {
        &self.diff
    }
    // Have to destroy the textures or else we will eat all the memory.
    // No setting textures after the canvas is dead, I guess? Not that there's any reason to do that anyway...
    pub fn set_comp(&mut self, tex: Texture) {
        unsafe {
            sdl2::sys::SDL_DestroyTexture(self.comp.raw());
        }
        self.comp = tex;
    }
    pub fn set_cur(&mut self, cur: Option<Texture>) {
        if let Some(c) = self.current.as_ref() {
            unsafe {
                sdl2::sys::SDL_DestroyTexture(c.raw());
            }
        };
        self.current = cur;
    }
    pub fn set_diff(&mut self, texture: Option<Texture>) {
        if let Some(d) = self.diff.as_ref() {
            unsafe {
                sdl2::sys::SDL_DestroyTexture(d.raw());
            }
        };
        self.diff = texture;
    }
}
