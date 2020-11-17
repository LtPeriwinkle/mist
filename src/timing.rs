//Functions used for timing and formatting of times

use std::time::{Instant, Duration};
use std::thread;
use sdl2::event::EventSender;

//Takes a number in milliseconds, divides it out into hours, minutes, seconds, and remaining millis
//then gets a time string with those values
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
				return format!("{}:{:02}:{:02}.{03}", hr, min, s, ms);
			} else { return format!("{}:{:02}.{:03}", min, s, ms); }

		} else { return format!("{}.{:03}", s, ms); }

	} else { return format!("{:03}", ms); }
}

pub struct TimeUpdateEvent {
	pub time: String
}

//uses a sleep-spin cycle to accurately update a time variable so that it is
//approximately the times when frames would occur in a 30fps game
pub fn time_30_fps(ev_sender: EventSender) {
	let mut loop_time: Instant;
	let mut time: u32 = 0; //u32 of milliseconds allows up to 49 day speedruns which should suffice
	loop {
		loop_time = Instant::now();
		thread::sleep(Duration::from_micros(32400)); //sleep less than the final desired duration
		while loop_time.elapsed().as_micros() < 33000 {
			thread::yield_now(); //yield_now basically tells the OS that the thread isn't doing anything
		}
		time += 33;
		ev_sender.push_custom_event(TimeUpdateEvent {time: ms_to_readable(time)}).unwrap();
		thread::sleep(Duration::from_micros(33400));
		while loop_time.elapsed().as_micros() < 67000 {
			thread::yield_now();
		}
		time += 34;
		ev_sender.push_custom_event(TimeUpdateEvent {time: ms_to_readable(time)}).unwrap();
		thread::sleep(Duration::from_micros(32400));
		while loop_time.elapsed().as_micros() < 100_000 {
			thread::yield_now();
		}
		time += 33;
		ev_sender.push_custom_event(TimeUpdateEvent {time: ms_to_readable(time)}).unwrap();
	}
}