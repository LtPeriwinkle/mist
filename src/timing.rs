//Functions and macro used for timing and formatting of times

use std::time::{Instant, Duration};
use std::thread;

macro_rules! pretty_time {
	($ms:ident) => {format!("{:03}", $ms)};
	($s:ident, $ms:ident) => {format!("{}.{:03}", $s, $ms)};
	($min:ident, $s:ident, $ms:ident) => {format!("{}:{:02}.{:03}", $min, $s, $ms)};
	($hr:ident, $min:ident, $s:ident, $ms:ident) => {format!("{}:{:02}:{:02}.{:03}", $hr, $min, $s, $ms)};
}

fn get_time_string(hr: u32, min: u32, s: u32, ms: u32) -> String {
	if hr != 0 {
		pretty_time!(hr, min, s, ms)
	} else if min != 0 {
		pretty_time!(min, s, ms)
	} else if s != 0 {
		pretty_time!(s, ms)
	} else {
		pretty_time!(ms)
	}
}

pub fn ms_to_readable(mut ms: u32) -> String {
	if ms >= 1000 {
		let remain_ms: u32 = ms % 1000;
		ms -= remain_ms;
		let mut s: u32 = ms / 1000;

		if s >= 60 {
			let remain_s = s % 60;
			s -= remain_s;
			let mut min = s / 60;

			if min >= 60 {
				let remain_min = min % 60;
				min -= remain_min;
				let hr = min / 60;
				return get_time_string(hr, remain_min, remain_s, remain_ms);
			} else { return get_time_string(0, min, remain_s, remain_ms); }

		} else { return get_time_string(0, 0, s, remain_ms); }

	} else { return get_time_string(0, 0, 0, ms); }
}

pub fn time_30_fps() {
	let mut loop_time: Instant;
	let mut time: u32 = 0;
	loop {
		loop_time = Instant::now();
		thread::sleep(Duration::from_micros(32400));
		while loop_time.elapsed().as_micros() < 33000 {
			thread::yield_now();
		}
		time += 33;
		println!("{}", ms_to_readable(time));
		thread::sleep(Duration::from_micros(33400));
		while loop_time.elapsed().as_micros() < 67000 {
			thread::yield_now();
		}
		time += 34;
		println!("{}", ms_to_readable(time));
		thread::sleep(Duration::from_micros(32400));
		while loop_time.elapsed().as_micros() < 100_000 {
			thread::yield_now();
		}
		time += 33;
		println!("{}", ms_to_readable(time));
	}
}