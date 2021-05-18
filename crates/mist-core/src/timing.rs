//! Functions used for formatting of times

/// Convert milliseconds into a readable time in the form HH:MM:SS.mmm
///
/// Optionally rounds to a possible 30hz value, i.e. 33ms, 67ms, etc.
///
/// # Arguments
///
/// * `ms` - the value to convert to string.
/// * `round` - whether to round to 30hz or not
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    // 3,611,111 ms should be 1 hour, 1 minute, 1 second, 111 ms
    fn test_readable() {
        assert_eq!(ms_to_readable(3_661_111, false), "1:01:01.111");
    }
    #[test]
    fn test_rounding_30() {
        assert_eq!(round_ms_30(500), 500);
        assert_eq!(round_ms_30(710), 700);
        assert_eq!(round_ms_30(645), 633);
        assert_eq!(round_ms_30(384), 367);
        assert_eq!(round_ms_30(399), 367);
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
