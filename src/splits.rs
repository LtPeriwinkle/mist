use sdl2::render::Texture;

pub struct Split<'a> {
    diff: Option<Texture<'a>>,
    name: Texture<'a>,
    comp: Texture<'a>,
    current: Option<Texture<'a>>,
}

impl<'a> Split<'a> {
    pub fn new(
        name: Texture<'a>,
        comp: Texture<'a>,
        diff: Option<Texture<'a>>,
        current: Option<Texture<'a>>,
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
    pub fn set_comp(&mut self, tex: Texture<'a>) {
        self.comp = tex;
    }
    pub fn cur(&self) -> &Option<Texture> {
        &self.current
    }
    pub fn set_cur(&mut self, cur: Option<Texture<'a>>) {
        self.current = cur;
    }
    pub fn set_diff(&mut self, texture: Option<Texture<'a>>) {
        self.diff = texture;
    }
    pub fn diff(&self) -> &Option<Texture<'a>> {
        &self.diff
    }
}
