// Functions used for formatting of times

// Takes a number in milliseconds, divides it out into hours, minutes, seconds, and remaining millis
// then gets a time string with those values
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

// quick and dirty solution for outputting valid 30fps times on pause/stop
fn round_ms_30(ms: u128) -> u128 {
    let mut rounded = ms;
    let mut hundreds = 0;
    while rounded >= 100 {
        rounded -= 100;
        hundreds += 100;
    }
    rounded = match rounded {
        0..=32 => 0,
        33..=66 => 33,
        67..=99 => 67,
        _ => 0,
    };
    rounded + hundreds
}

pub fn split_time_sum(ms_vec: Vec<u128>) -> Vec<String> {
	let mut total = 0;
	let mut vec = vec![];
	for num in ms_vec {
		total += num;
		vec.push(ms_to_readable(total, false));
	}
	vec
}
