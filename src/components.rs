use sdl2::pixels::Color;

pub const SPLITS_ON_SCREEN: usize = 8;
pub static MAKING_UP_TIME: Color = Color::RGB(255, 60, 60);
pub static LOSING_TIME: Color = Color::RGB(60, 255, 60);

#[derive(Debug)]
pub enum TimerState {
	OffsetCountdown {amt: u128},
	Running {timestamp: u32},
	Paused {time: u128, time_str: String},
	NotStarted {time_str: String},
	Finished {time_str: String}
}
