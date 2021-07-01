//! Functions used for formatting of times

/// Convert milliseconds into a readable time in the form HH:MM:SS.mmm
///
/// Optionally rounds to a possible 30hz value, i.e. 33ms, 67ms, etc.
///
/// # Arguments
///
/// * `ms` - the value to convert to string.
/// * `round` - `Some(value)` to round to `value` frames/sec. None for no rounding
pub fn ms_to_readable(mut ms: u128, round: Option<u128>) -> String {
    if round.is_some() {
        ms = round_ms(round.unwrap(), ms);
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

/// Create the readable time for a time differences.
///
/// Prefixes with `+` for lost time and `-` for gained time.
///
/// Passing a negative value of `ms` specifies gained time and returns a `-` prefixed string.
///
/// Truncates decimals at the tenths place.
pub fn diff_text(mut ms: i128) -> String {
    let pre: char;
    if ms < 0 {
        ms *= -1;
        pre = '-';
    } else {
        pre = '+';
    }
    let mut tenths = ms / 100;
    let mut full_s: i128;
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
}

/// Creates the text for times of splits.
///
/// Essentially the same as [ms_to_readable] but truncates at tenths place.
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

/// Gets the sums of elements in a vec
///
/// Returns a Vec with the sums of every element up to that point in it.
///
/// For example, input of [6, 7, 8] returns [6, 13, 21].
pub fn split_time_sum(ms_vec: &Vec<u128>) -> Vec<u128> {
    let mut total = 0;
    let mut vec = vec![];
    for num in ms_vec {
        total += num;
        vec.push(total);
    }
    vec
}

fn round_ms(frames: u128, ms: u128) -> u128 {
    let hundreds = ms / 100;
    let mut rem = ms % 100;
    println!("{:?}", gen_round_values(frames));
    let rounds = gen_round_values(frames);
    for val in &rounds {
        println!("{}", val);
        if rem <= *val {
            rem = val - rounds[0];
            println!("{}", rem);
            if *val % 10 >= 5 && rounds[0] % 10 < 5 {
                rem -= 1;
            }
            println!("{}", rem);
            break;
        }
    }
    rem + (hundreds * 100)
}

fn gen_round_values(frames: u128) -> Vec<u128> {
    let frame = 1000 / frames;
    let mut sum = frame;
    let mut ret = vec![];
    while sum < 100 {
        if sum % 10 >= 5 {
            sum += 1;
        }
        ret.push(sum);
        sum += frame;
    }
    ret.push(sum);
    ret
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    // 3,611,111 ms should be 1 hour, 1 minute, 1 second, 111 ms
    fn test_readable() {
        assert_eq!(ms_to_readable(3_661_111, None), "1:01:01.111");
    }
    #[test]
    fn test_rounding_30() {
        assert_eq!(round_ms(30, 500), 500);
        assert_eq!(round_ms(30, 710), 700);
        assert_eq!(round_ms(30, 645), 633);
        assert_eq!(round_ms(30, 384), 367);
        assert_eq!(round_ms(30, 399), 367);
    }
    #[test]
    fn test_rounding_60() {
        assert_eq!(round_ms(60, 500), 500);
        assert_eq!(round_ms(60, 710), 700);
        assert_eq!(round_ms(60, 460), 450);
        assert_eq!(round_ms(60, 645), 633);
        assert_eq!(round_ms(60, 384), 383);
        assert_eq!(round_ms(60, 399), 383);
    }
    #[test]
    fn test_sum() {
        assert_eq!(split_time_sum(&vec![6, 7, 8]), vec![6, 13, 21]);
        assert_eq!(split_time_sum(&vec![0]), vec![0]);
    }
    #[test]
    fn test_diff() {
        assert_eq!(diff_text(3_661_111), "+1:01:01.1");
        assert_eq!(diff_text(-3_661_111), "-1:01:01.1");
    }
    #[test]
    fn test_split() {
        assert_eq!(split_time_text(3_661_111), "1:01:01.1");
    }
}
