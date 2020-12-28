// split struct and methods
use sdl2::render::Texture;

// this holds a lot of information about both what to render and how to compare to it
pub struct Split<'a> {
    pb_time: u128,
    gold_time: u128,
    diff: i128,
    diff_texture: Option<Texture<'a>>,
    name_texture: Texture<'a>,
    pb_texture: Texture<'a>,
    current_texture: Option<Texture<'a>>,
}

// requires lifetime specifier because textures aren't allowed to outlive texture_creators
impl<'a> Split<'a> {
    pub fn new(
        pb_time: u128,
        gold_time: u128,
        diff: i128,
        diff_texture: Option<Texture<'a>>,
        name_texture: Texture<'a>,
        pb_texture: Texture<'a>,
        current_texture: Option<Texture<'a>>,
    ) -> Self {
        Self {
            pb_time,
            gold_time,
            diff,
            diff_texture,
            name_texture,
            pb_texture,
            current_texture,
        }
    }
    pub fn time(&self) -> u128 {
        self.pb_time
    }
    pub fn set_time(&mut self, time: u128) {
        self.pb_time = time;
    }
    pub fn name(&self) -> &Texture {
        &self.name_texture
    }
    pub fn pb(&self) -> &Texture {
        &self.pb_texture
    }
    pub fn set_pb(&mut self, tex: Texture<'a>) {
        self.pb_texture = tex;
    }
    pub fn cur(&self) -> &Option<Texture> {
        &self.current_texture
    }
    pub fn set_cur(&mut self, cur: Option<Texture<'a>>) {
        self.current_texture = cur;
    }
    pub fn set_diff(&mut self, diff: i128, texture: Option<Texture<'a>>) {
        self.diff = diff;
        self.diff_texture = texture;
    }
    pub fn diff(&self) -> i128 {
        self.diff
    }
    pub fn diff_texture(&self) -> &Option<Texture<'a>> {
        &self.diff_texture
    }
    pub fn gold(&self) -> u128 {
        self.gold_time
    }
    pub fn set_gold(&mut self, gold: u128) {
        self.gold_time = gold;
    }
}
