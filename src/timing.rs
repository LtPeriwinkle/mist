//Functions used for timing and formatting of times

//Takes a number in milliseconds, divides it out into hours, minutes, seconds, and remaining millis
//then gets a time string with those values
pub fn ms_to_readable(mut ms: u128) -> String {
	if ms >= 1000 {
		let remain_ms = ms % 1000;
		ms -= remain_ms;
		let mut s = ms / 1000;

		if s >= 60 {
			let remain_s = s % 60;
			s -= remain_s;
			let mut min = s / 60;

			if min >= 60 {
				let remain_min = min % 60;
				min -= remain_min;
				let hr = min / 60;
				return format!("{}:{:02}:{:02}.{:03}", hr, remain_min, remain_s, remain_ms);
			} else { return format!("{}:{:02}.{:03}", min, remain_s, remain_ms); }

		} else { return format!("{}.{:03}", s, remain_ms); }

	} else { return format!("0.{:03}", ms); }
}
