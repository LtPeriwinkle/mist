use sdl2::pixels::Color;

pub const SPLITS_ON_SCREEN: usize = 8; // max splits allowed on screen
pub static MAKING_UP_TIME: Color = Color::RGB(255, 60, 60); // color used when behind but gaining
pub static LOSING_TIME: Color = Color::RGB(135, 255, 135); // color used when ahead but losing

// state of timer, might implement real state switching eventually
#[derive(Debug)]
pub enum TimerState {
    OffsetCountdown { amt: u128 },
    Running { timestamp: u128 },
    Paused { time: u128, time_str: String },
    NotStarted { time_str: String },
    Finished { time_str: String },
}
