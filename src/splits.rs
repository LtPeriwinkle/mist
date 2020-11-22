// will do something eventually
pub fn get_splits() -> Vec<&'static str> {
    vec![
        "Something",
        "else",
        "words",
        "text",
        "split 5 idk",
        "q",
        "asdf",
        "words 2",
        "no",
        "yes",
        "another one",
    ]
}

pub fn get_split_times(start: usize, end: usize) -> Vec<u128> {
    let mut vec = vec![];
    let mut index = start;
    while index < end {
        vec.push(get_time(index));
        index += 1;
    }
    vec
}

fn get_time(index: usize) -> u128 {
    let time_list = [1000, 2000, 3000, 4000, 5000, 6000, 7000, 8000, 9000, 10000, 11000];
    return time_list[index];
}