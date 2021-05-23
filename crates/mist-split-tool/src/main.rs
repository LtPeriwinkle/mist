use fltk::{app, button, dialog, draw, input, table, window::*};
use mist_core::{parse::LssParser, parse::MsfParser, Run};
use std::convert::TryInto;
use std::sync::Mutex;
use tinyfiledialogs as tfd;

use lazy_static::lazy_static;

static HEADERS: [&'static str; 3] = ["Split Name", "Personal Best", "Gold"];

lazy_static! {
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

fn ms_to_readable(mut ms: u128) -> String {
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
    } else {
        return format!("0.{:03}", ms);
    }
}

fn str_to_ms(tm: String) -> u128 {
    unsafe {
        ILLEGAL = false;
    }
    let mut ms: u128 = 0;
    fn alert(_: std::num::ParseIntError) -> u128  {
        dialog::alert_default("invalid time entered");
        unsafe {
            ILLEGAL = true;
        }
        0
    }
    let split: Vec<&str> = tm.split(':').collect();
    if split.len() == 2 {
        ms += split[0].parse::<u128>().unwrap_or_else(alert) * 60000;
        let split2: Vec<&str> = split[1].split('.').collect();
        ms += split2[0].parse::<u128>().unwrap_or_else(alert) * 1000;
        ms += split2[1].parse::<u128>().unwrap_or_else(alert);
    } else if split.len() == 3 {
        ms += split[0].parse::<u128>().unwrap_or_else(alert) * 3600000;
        ms += split[1].parse::<u128>().unwrap_or_else(alert) * 60000;
        let split2: Vec<&str> = split[2].split('.').collect();
        ms += split2[0].parse::<u128>().unwrap_or_else(alert) * 1000;
        ms += split2[1].parse::<u128>().unwrap_or_else(alert);
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
        None => return,
    }
    let app = app::App::default();
    let mut win = Window::default()
        .with_size(510, 600)
        .center_screen()
        .with_label("mist split editor");
    let mut table = table::Table::new(5, 50, 503, 550, "");
    let og_len: u32 = RUN.lock().unwrap().splits().len().try_into().unwrap();
    table.set_rows(og_len);
    table.set_row_header(true);
    table.set_cols(3);
    table.set_col_header(true);
    table.set_col_width(0, 180);
    table.set_col_width(1, 140);
    table.set_col_width(2, 140);
    table.end();
    let mut save_button = button::Button::new(423, 25, 80, 25, "save file");
    let mut add_button = button::Button::new(342, 25, 80, 25, "add split");
    let mut sub_button = button::Button::new(261, 25, 80, 25, "remove split");
    let mut open_button = button::Button::new(180, 25, 80, 25, "open file");
    win.make_resizable(false);
    win.end();
    win.show();
    let mut tbl = table.clone();
    open_button.set_callback(move || {
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
        fltk::TableExt::clear(&mut tbl);
        tbl.set_rows(og_len);
        tbl.set_row_header(true);
        tbl.set_cols(3);
        tbl.set_col_header(true);
        tbl.set_col_width(0, 180);
        tbl.set_col_width(1, 140);
        tbl.set_col_width(2, 140);
    });
    save_button.set_callback(move || {
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
    table.draw_cell2(move |t, ctx, row, col, x, y, w, h| {
        match ctx {
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
                        inp.set_callback2(move |input| {
                            if (row as usize) >= VECS.lock().unwrap().2.len() {
                                VECS.lock().unwrap().2.push(input.value())
                            } else {
                                VECS.lock().unwrap().2[row as usize] = input.value();
                            }
                        })
                    } else if col == 1 {
                        if (row as usize) < VECS.lock().unwrap().0.len() {
                            inp.set_value(&ms_to_readable(
                                VECS.lock().unwrap().0[row as usize],
                            ));
                        }
                        inp.set_callback2(move |input| {
                            if (row as usize) >= VECS.lock().unwrap().0.len() {
                                VECS.lock().unwrap().0.push(str_to_ms(input.value()))
                            } else {
                                VECS.lock().unwrap().0[row as usize] = str_to_ms(input.value());
                            }
                        })
                    } else if col == 2 {
                        if (row as usize) < VECS.lock().unwrap().1.len() {
                            inp.set_value(&ms_to_readable(
                                VECS.lock().unwrap().1[row as usize],
                            ));
                        }
                        inp.set_callback2(move |input| {
                            if (row as usize) >= VECS.lock().unwrap().1.len() {
                                VECS.lock().unwrap().1.push(str_to_ms(input.value()))
                            } else {
                                VECS.lock().unwrap().1[row as usize] = str_to_ms(input.value());
                            }
                        })
                    }
                } else {
                    if col == 0 {
                        inp.set_callback2(move |input| {
                            if (row as usize) > VECS.lock().unwrap().2.len() {
                                VECS.lock().unwrap().2.push(input.value())
                            } else {
                                VECS.lock().unwrap().2.insert(row as usize, input.value());
                            }
                        })
                    } else if col == 1 {
                        inp.set_callback2(move |input| {
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
                        inp.set_callback2(move |input| {
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
        }
    });
    let mut table1 = table.clone();
    add_button.set_callback(move || {
        table.set_rows(table.rows() + 1);
        table.redraw();
    });
    sub_button.set_callback(move || {
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
