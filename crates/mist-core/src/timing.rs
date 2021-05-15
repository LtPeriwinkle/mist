// Functions used for formatting of times

pub fn ms_to_readable(mut ms: u128, round: bool) -> String {
    if round {
        ms = round_ms_30(ms);
    }
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
            } else {
                return format!("{}:{:02}.{:03}", min, remain_s, remain_ms);
            }
        } else {
            return format!("{}.{:03}", s, remain_ms);
        }
    }
    return format!("0.{:03}", ms);
}

pub fn diff_text(mut ms: i128) -> String {
    let negative = if ms < 0 {
        ms *= -1;
        true
    } else {
        false
    };
    let pre: char;
    if negative {
        pre = '-';
    } else {
        pre = '+';
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
                return format!("{}{}:{:02}:{:02}.{}", pre, hr, min, full_s, tenths);
            } else {
                return format!("{}{}:{:02}.{}", pre, min, full_s, tenths);
            }
        } else {
            return format!("{}{}.{}", pre, full_s, tenths);
        }
    } else {
        return format!("{}0.{}", pre, tenths);
    }
}

pub fn split_time_text(ms: u128) -> String {
    let mut tenths = ms / 100;
    let mut full_s: u128;
    if tenths > 10 {
        full_s = tenths / 10;
        tenths -= full_s * 10;
        if full_s >= 60 {
            let mut min = full_s / 60;
            full_s -= min * 60;
            if min >= 60 {
                let hr = min / 60;
                min -= hr * 60;
                return format!("{}:{:02}:{:02}.{}", hr, min, full_s, tenths);
            } else {
                return format!("{}:{:02}.{}", min, full_s, tenths);
            }
        } else {
            return format!("{}.{}", full_s, tenths);
        }
    } else {
        return format!("0.{}", tenths);
    }
}

fn round_ms_30(ms: u128) -> u128 {
    let hundreds = ms / 100;
    let mut rounded = ms % 100;
    rounded = match rounded {
        0..=32 => 0,
        33..=66 => 33,
        67..=99 => 67,
        _ => 0,
    };
    rounded + (hundreds * 100)
}

pub fn split_time_sum(ms_vec: &Vec<u128>) -> Vec<u128> {
    let mut total = 0;
    let mut vec = vec![];
    for num in ms_vec {
        total += num;
        vec.push(total);
    }
    vec
}
