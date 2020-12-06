// Functions used for formatting of times

// Takes a number in milliseconds, divides it out into hours, minutes, seconds, and remaining millis
// then gets a time string with those values
// if it is told to round the times to 30fps, calls round_ms_30 on the remaining millis
pub fn ms_to_readable(mut ms: u128, round: bool) -> String {
    if ms >= 1000 {
        let mut remain_ms = ms % 1000;
        ms -= remain_ms;
        let mut s = ms / 1000;

        if round {
            remain_ms = round_ms_30(remain_ms);
        }

        if s >= 60 {
            let remain_s = s % 60;
            s -= remain_s;
            let mut min = s / 60;

            if min >= 60 {
                let remain_min = min % 60;
                min -= remain_min;
                let hr = min / 60;
                return format!("{}:{:02}:{:02}.{:03}", hr, remain_min, remain_s, remain_ms);
            } else {
                return format!("{}:{:02}.{:03}", min, remain_s, remain_ms);
            }
        } else {
            return format!("{}.{:03}", s, remain_ms);
        }
    } else {
        if round {
            return format!("0.{:03}", round_ms_30(ms));
        } else {
            return format!("0.{:03}", ms);
        }
    }
}

pub fn diff_text(mut ms: i128) -> String {
	let mut negative = false;
	if ms < 0 {
		negative = true;
		ms *= -1;
	}
	let mut tenths = ms / 100;
	let mut full_s: i128;
	if tenths > 10 {
		full_s = tenths / 10;
		tenths -= full_s * 10;
		if full_s >= 60 {
			let mut min = full_s / 60;
			full_s -= min * 60;
			if min >= 60 {
				let hr = min / 60;
				min -= hr * 60;
				if negative {
					return format!("-{}:{}:{}.{}", hr, min, full_s, tenths);
				} else {
					return format!("{}:{}:{}.{}", hr, min, full_s, tenths);
				}
			} else {
    				if negative {
					return format!("-{}:{}.{}", min, full_s, tenths);
    				} else {
					return format!("{}:{}.{}", min, full_s, tenths);
    				}
			}
		} else {
    			if negative {
				return format!("-{}.{}", full_s, tenths);
    			} else {
				return format!("{}.{}", full_s, tenths);
    			}
		}
	} else {
    		if negative {
			return format!("-0.{}", tenths);
    		} else {
			return format!("0.{}", tenths);
    		}
	}
}

// quick and dirty solution for outputting valid 30fps times on pause/stop
fn round_ms_30(ms: u128) -> u128 {
    let mut rounded = ms;
    let hundreds = rounded / 100;
    rounded -= hundreds * 100;
    rounded = match rounded {
        0..=32 => 0,
        33..=66 => 33,
        67..=99 => 67,
        _ => 0,
    };
    rounded + (hundreds * 100)
}

// add up the times from split files to get the total real time
pub fn split_time_sum(ms_vec: &Vec<u128>) -> Vec<u128> {
    let mut total = 0;
    let mut vec = vec![];
    for num in ms_vec {
        total += num;
        vec.push(total);
    }
    vec
}
