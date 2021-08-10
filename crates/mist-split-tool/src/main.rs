use fltk::{app, button, dialog, draw, enums::{FrameType, Font, Color, Align}, input, prelude::*, table, window::*};
use lazy_static::lazy_static;
use mist_core::{parse::LssParser, parse::MsfParser, timing::ms_to_readable, Run};
use regex::Regex;
use std::convert::TryInto;
use std::sync::Mutex;
use tinyfiledialogs as tfd;

static HEADERS: [&'static str; 3] = ["Split Name", "Personal Best", "Gold"];

lazy_static! {
    static ref HOURS: Regex = Regex::new(r"^(\d+):([0-5]\d):([0-5]\d)(?:\.(\d{1,3}))?$").unwrap();
    static ref MINS: Regex = Regex::new(r"^([0-5]?\d):([0-5]\d)(?:\.(\d{1,3}))?$").unwrap();
    static ref SECS: Regex = Regex::new(r"^([0-5]?\d)(?:\.(\d{1,3}))?$").unwrap();
    static ref RUN: Mutex<Run> = Mutex::new(Run::empty());
    static ref VECS: Mutex<(Vec<u128>, Vec<u128>, Vec<String>)> =
        Mutex::new((vec![], vec![], vec![]));
}

static mut ILLEGAL: bool = false;

fn open_split_file() -> Option<String> {
    tfd::open_file_dialog(
        "Open a split file (msf or lss)",
        "",
        Some((&["*.msf", "*.lss"], "")),
    )
}

fn get_save_as() -> Option<String> {
    match tfd::save_file_dialog_with_filter("Save to MSF file", "", &["*.msf"], "mist split files")
    {
        Some(mut p) => {
            if p.ends_with(".msf") {
                Some(p)
            } else {
                p.push_str(".msf");
                Some(p)
            }
        }
        None => None,
    }
}

fn str_to_ms(tm: String) -> u128 {
    let mut ms: u128 = 0;
    if HOURS.is_match(&tm) {
        let caps = HOURS.captures_iter(&tm).next().unwrap();
        ms = (caps[1].parse::<u128>()
            .unwrap()
            * 3600000)
            + (caps[2]
                .parse::<u128>()
                .unwrap()
                * 60000)
            + (caps[3]
                .parse::<u128>()
                .unwrap()
                * 1000)
            + caps
                .get(4)
                .map_or("0", |m| m.as_str())
                .parse::<u128>()
                .unwrap();
    } else if MINS.is_match(&tm) {
        let caps = MINS.captures_iter(&tm).next().unwrap();
        ms = (caps[1]
            .parse::<u128>()
            .unwrap()
            * 60000)
            + (caps[2]
                .parse::<u128>()
                .unwrap()
                * 1000)
            + caps
                .get(3)
                .map_or("0", |m| m.as_str())
                .parse::<u128>()
                .unwrap();
    } else if SECS.is_match(&tm) {
        let caps = SECS.captures_iter(&tm).next().unwrap();
        ms = (caps[1].parse::<u128>().unwrap() * 1000) + caps.get(2).map_or("0", |m| m.as_str()).parse::<u128>().unwrap();
    }
    return ms;
}

fn main() {
    let path = open_split_file();
    let mut save_path: String = "".to_string();
    match path {
        Some(ref p) => {
            let run: Run;
            if p.ends_with(".msf") {
                let parser = MsfParser::new();
                let f = std::fs::File::open(p).unwrap();
                run = parser.parse(std::io::BufReader::new(f)).unwrap();
                *VECS.lock().unwrap() = (
                    run.pb_times().to_owned(),
                    run.gold_times().to_owned(),
                    run.splits().to_owned(),
                );
                *RUN.lock().unwrap() = run;
            } else {
                let f = std::fs::File::open(p).unwrap();
                let mut parser = LssParser::new(std::io::BufReader::new(f));
                run = parser.parse();
                *VECS.lock().unwrap() = (
                    run.pb_times().to_owned(),
                    run.gold_times().to_owned(),
                    run.splits().to_owned(),
                );
                *RUN.lock().unwrap() = run;
            }
        }
        None => *RUN.lock().unwrap() = Run::empty(),
    }
    let app = app::App::default();
    let mut win = Window::default()
        .with_size(510, 635)
        .center_screen()
        .with_label("mist split editor");
    let mut table = table::Table::new(5, 85, 503, 550, "");
    let og_len: i32 = RUN.lock().unwrap().splits().len().try_into().unwrap();
    let og_len = if og_len == 0 {1} else {og_len};
    table.set_rows(og_len);
    table.set_row_header(true);
    table.set_cols(3);
    table.set_col_header(true);
    table.set_col_width(0, 180);
    table.set_col_width(1, 140);
    table.set_col_width(2, 140);
    table.end();
    let mut save_button = button::Button::new(423, 60, 80, 25, "save file");
    let mut add_button = button::Button::new(342, 60, 80, 25, "add split");
    let mut sub_button = button::Button::new(261, 60, 80, 25, "remove split");
    let mut open_button = button::Button::new(180, 60, 80, 25, "open file");
    let mut title_inp = input::Input::new(100, 5, 180, 25, "Category Title: ");
    let mut cat_inp = input::Input::new(100, 30, 180, 25, "Game Title: ");
    win.make_resizable(false);
    win.end();
    win.show();
    cat_inp.set_callback(|inp| {RUN.lock().unwrap().set_category(inp.value())});
    title_inp.set_callback(|inp| {RUN.lock().unwrap().set_game_title(inp.value())});
    cat_inp.set_value(RUN.lock().unwrap().category());
    title_inp.set_value(RUN.lock().unwrap().game_title());
    let mut tbl = table.clone();
    open_button.set_callback(move |_| {
        let path = open_split_file();
        match path {
            Some(ref p) => {
                if p.ends_with(".msf") {
                    let parser = MsfParser::new();
                    let f = std::fs::File::open(p).unwrap();
                    let run = parser.parse(std::io::BufReader::new(f)).unwrap();
                    *VECS.lock().unwrap() = (
                        run.pb_times().to_owned(),
                        run.gold_times().to_owned(),
                        run.splits().to_owned(),
                    );
                    *RUN.lock().unwrap() = run;
                } else {
                    let f = std::fs::File::open(p).unwrap();
                    let mut parser = LssParser::new(std::io::BufReader::new(f));
                    let run = parser.parse();
                    *VECS.lock().unwrap() = (
                        run.pb_times().to_owned(),
                        run.gold_times().to_owned(),
                        run.splits().to_owned(),
                    );
                    *RUN.lock().unwrap() = run;
                }
            }
            None => return,
        }
        TableExt::clear(&mut tbl);
        tbl.set_rows(og_len);
        tbl.set_row_header(true);
        tbl.set_cols(3);
        tbl.set_col_header(true);
        tbl.set_col_width(0, 180);
        tbl.set_col_width(1, 140);
        tbl.set_col_width(2, 140);
    });
    save_button.set_callback(move |_| {
        let vecs = VECS.lock().unwrap();
        let mut run = RUN.lock().unwrap();
        run.set_pb_times(&vecs.0);
        run.set_gold_times(&vecs.1);
        run.set_splits(&vecs.2);
        run.set_pb(vecs.1.iter().sum());
        // fill sum times with empty ones until i figure out how i want to handle it
        run.set_sum_times(&vecs.2.iter().map(|_| (0u128, 0u128)).collect());
        unsafe {
            if !ILLEGAL {
                let parser = MsfParser::new();
                let f: std::fs::File;
                if save_path != "".to_string() {
                    f = std::fs::File::create(save_path.clone()).unwrap();
                    parser.write(&run, f).unwrap();
                } else {
                    let name = get_save_as();
                    match name {
                        Some(ref p) => {
                            if p != "()" {
                                f = std::fs::File::create(p).unwrap();
                                parser.write(&run, f).unwrap();
                                save_path = p.to_owned();
                            }
                        }
                        None => {}
                    }
                }
            } else {
                dialog::alert_default("invalid time(s) entered");
            }
        }
    });
    table.draw_cell(move |t, ctx, row, col, x, y, w, h| match ctx {
        table::TableContext::StartPage => draw::set_font(Font::Helvetica, 14),
        table::TableContext::ColHeader => {
            draw::push_clip(x, y, w, h);
            draw::draw_box(FrameType::ThinUpBox, x, y, w, h, Color::FrameDefault);
            draw::set_draw_color(Color::Black);
            draw::draw_text2(HEADERS[col as usize], x, y, w, h, Align::Center);
            draw::pop_clip();
        }
        table::TableContext::RowHeader => {
            draw::push_clip(x, y, w, h);
            draw::draw_box(FrameType::ThinUpBox, x, y, w, h, Color::FrameDefault);
            draw::set_draw_color(Color::Black);
            draw::draw_text2(&format!("{}", row + 1), x, y, w, h, Align::Center);
            draw::pop_clip();
        }
        table::TableContext::Cell => {
            let mut inp = input::Input::new(x, y, w, h, "");
            if row < ((VECS.lock().unwrap().2.len()) as i32) {
                if col == 0 {
                    inp.set_value(&VECS.lock().unwrap().2[row as usize]);
                    inp.set_callback(move |input| {
                        if (row as usize) >= VECS.lock().unwrap().2.len() {
                            VECS.lock().unwrap().2.push(input.value())
                        } else {
                            VECS.lock().unwrap().2[row as usize] = input.value();
                        }
                    })
                } else if col == 1 {
                    if (row as usize) < VECS.lock().unwrap().0.len() {
                        inp.set_value(&ms_to_readable(VECS.lock().unwrap().0[row as usize], None));
                    }
                    inp.set_callback(move |input| {
                        if (row as usize) >= VECS.lock().unwrap().0.len() {
                            VECS.lock().unwrap().0.push(str_to_ms(input.value()))
                        } else {
                            VECS.lock().unwrap().0[row as usize] = str_to_ms(input.value());
                        }
                    })
                } else if col == 2 {
                    if (row as usize) < VECS.lock().unwrap().1.len() {
                        inp.set_value(&ms_to_readable(VECS.lock().unwrap().1[row as usize], None));
                    }
                    inp.set_callback(move |input| {
                        if (row as usize) >= VECS.lock().unwrap().1.len() {
                            VECS.lock().unwrap().1.push(str_to_ms(input.value()))
                        } else {
                            VECS.lock().unwrap().1[row as usize] = str_to_ms(input.value());
                        }
                    })
                }
            } else {
                if col == 0 {
                    inp.set_callback(move |input| {
                        if (row as usize) > VECS.lock().unwrap().2.len() {
                            VECS.lock().unwrap().2.push(input.value())
                        } else {
                            VECS.lock().unwrap().2.insert(row as usize, input.value());
                        }
                    })
                } else if col == 1 {
                    inp.set_callback(move |input| {
                        if (row as usize) > VECS.lock().unwrap().0.len() {
                            VECS.lock().unwrap().0.push(str_to_ms(input.value()))
                        } else {
                            VECS.lock()
                                .unwrap()
                                .0
                                .insert(row as usize, str_to_ms(input.value()));
                        }
                    })
                } else if col == 2 {
                    inp.set_callback(move |input| {
                        if (row as usize) > VECS.lock().unwrap().1.len() {
                            VECS.lock().unwrap().1.push(str_to_ms(input.value()))
                        } else {
                            VECS.lock()
                                .unwrap()
                                .1
                                .insert(row as usize, str_to_ms(input.value()));
                        }
                    })
                }
            }
            t.add(&inp);
        }
        _ => {}
    });
    let mut table1 = table.clone();
    add_button.set_callback(move |_| {
        table.set_rows(table.rows() + 1);
        table.redraw();
    });
    sub_button.set_callback(move |_| {
        let new_rows = table1.rows() - 1;
        if (new_rows as usize) < VECS.lock().unwrap().0.len() {
            VECS.lock().unwrap().0.pop();
        }
        if (new_rows as usize) < VECS.lock().unwrap().1.len() {
            VECS.lock().unwrap().1.pop();
        }
        if (new_rows as usize) < VECS.lock().unwrap().2.len() {
            VECS.lock().unwrap().2.pop();
        }
        table1.set_rows(new_rows);
        table1.redraw();
    });
    app.run().unwrap();
}
