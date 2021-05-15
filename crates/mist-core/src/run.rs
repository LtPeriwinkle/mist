#[derive(Debug)]
pub struct Run {
    game_title: String,
    category: String,
    offset: Option<u128>,
    pb: u128,
    splits: Vec<String>,
    pb_times: Vec<u128>,
    gold_times: Vec<u128>,
    sum_times: Vec<(u128, u128)>,
}

impl Run {
    pub fn empty() -> Self {
        Run {
            game_title: "".to_owned(),
            category: "".to_owned(),
            offset: None,
            pb: 0,
            splits: vec![],
            pb_times: vec![],
            gold_times: vec![],
            sum_times: vec![],
        }
    }
    pub fn game_title(&self) -> &str {
        &self.game_title
    }
    pub fn category(&self) -> &str {
        &self.category
    }
    pub fn offset(&self) -> Option<u128> {
        self.offset
    }
    pub fn pb(&self) -> u128 {
        self.pb
    }
    pub fn splits(&self) -> &Vec<String> {
        &self.splits
    }
    pub fn pb_times(&self) -> &Vec<u128> {
        &self.pb_times
    }
    pub fn gold_times(&self) -> &Vec<u128> {
        &self.gold_times
    }
    pub fn sum_times(&self) -> &Vec<(u128, u128)> {
        &self.sum_times
    }
    pub fn set_game_title<S>(&mut self, new: S)
    where
        S: ToString,
    {
        self.game_title = new.to_string();
    }
    pub fn set_category<S>(&mut self, new: S)
    where
        S: ToString,
    {
        self.category = new.to_string();
    }
    pub fn set_offset(&mut self, new: Option<u128>) {
        self.offset = new;
    }
    pub fn set_pb(&mut self, new: u128) {
        self.pb = new;
    }
    pub fn set_splits(&mut self, new: &Vec<String>) {
        self.splits = new.to_owned();
    }
    pub fn set_gold_times(&mut self, new: &Vec<u128>) {
        self.gold_times = new.to_owned();
    }
    pub fn set_sum_times(&mut self, new: &Vec<(u128, u128)>) {
        self.sum_times = new.to_owned();
    }
}
