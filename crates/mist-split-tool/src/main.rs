use fltk::{app, button, draw, input, table, window::*};
use mist_run_utils::run::Run;
use std::convert::TryInto;
use std::env::current_dir;
use std::sync::Mutex;
use tinyfiledialogs as tfd;

use lazy_static::lazy_static;

static HEADERS: [&'static str; 3] = ["Split Name", "Personal Best", "Gold"];

lazy_static! {
    static ref RUN: Mutex<Run> = Mutex::new(Run::default());
}

fn open_split_file() -> Option<String> {
    let cwd = current_dir().unwrap();
    let mut dir = cwd.to_string_lossy();
    dir.to_mut().push('/');
    return tfd::open_file_dialog(
        "Open a split file (msf or lss)",
        &dir,
        Some((&["*.msf", "*.lss"], "")),
    );
}

fn get_save_as() -> Option<String> {
    let cwd = current_dir().unwrap();
    let mut dir = cwd.to_string_lossy();
    dir.to_mut().push('/');
    return tfd::save_file_dialog_with_filter("Save to MSF file", &dir, &["*.msf"], "mist split files");
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
    let mut ms: u128 = 0;
    let split: Vec<&str> = tm.split(':').collect();
    if split.len() == 2 {
        ms += split[0].parse::<u128>().unwrap() * 60000;
        let split2: Vec<&str> = split[1].split('.').collect();
        ms += split2[0].parse::<u128>().unwrap() * 1000;
        ms += split2[1].parse::<u128>().unwrap();
    } else {
        ms += split[0].parse::<u128>().unwrap() * 3600000;
        ms += split[1].parse::<u128>().unwrap() * 60000;
        let split2: Vec<&str> = split[2].split('.').collect();
        ms += split2[0].parse::<u128>().unwrap() * 1000;
        ms += split2[1].parse::<u128>().unwrap();
    }
    return ms;
}

fn main() {
    let path = open_split_file();
    let req_file: bool;
    match path {
        Some(ref p) => {
            if p.ends_with(".msf") {
                req_file = false;
                *RUN.lock().unwrap() = Run::from_msf_file(&p).unwrap();
            } else {
                req_file = true;
                *RUN.lock().unwrap() = Run::from_lss_file(&p).unwrap();
            }
        }
        None => {
            std::process::exit(0);
        }
    }
    let app = app::App::default();
    let mut win = Window::default()
        .with_size(510, 600)
        .center_screen()
        .with_label("mist split editor");
    let mut table = table::Table::new(5, 50, 503, 550, "Split Editor");
    table.set_rows(RUN.lock().unwrap().split_names().len().try_into().unwrap());
    table.set_row_header(true);
    table.set_cols(3);
    table.set_col_header(true);
    table.set_col_width(0, 180);
    table.set_col_width(1, 140);
    table.set_col_width(2, 140);
    table.end();
    let mut save_button = button::Button::new(423, 25, 80, 25, "save file");
    win.make_resizable(false);
    win.end();
    win.show();
    save_button.set_callback(move || {
        if !req_file {
            RUN.lock().unwrap().save_msf(path.as_ref().unwrap());
        } else {
            let name = get_save_as();
            println!("{:?}", name);
            match name {
                Some(p) => if p != "()" {RUN.lock().unwrap().save_msf(&p)},
                None => {}
            }
        }
    });
    table.draw_cell2(move |t, ctx, row, col, x, y, w, h| {
        match ctx {
            table::TableContext::StartPage => draw::set_font(Font::Courier, 14),
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
                if col == 0 {
                    inp.set_value(&RUN.lock().unwrap().split_names()[row as usize]);
                    inp.set_callback2(move |input| {
                        RUN.lock().unwrap().set_name(input.value(), row as usize);
                    })
                } else if col == 1 {
                    inp.set_value(&ms_to_readable(
                        RUN.lock().unwrap().get_times()[row as usize],
                    ));
                    inp.set_callback2(move |input| {
                        RUN.lock()
                            .unwrap()
                            .set_time(str_to_ms(input.value()), row as usize);
                    })
                } else if col == 2 {
                    inp.set_value(&ms_to_readable(
                        RUN.lock().unwrap().get_golds()[row as usize],
                    ));
                    inp.set_callback2(move |input| {
                        RUN.lock()
                            .unwrap()
                            .set_gold_time(row as usize, str_to_ms(input.value()));
                    })
                }
                t.add(&inp);
            }
            _ => {}
        }
    });
    while table.children() == 0 {
        app::wait();
    }
    app.run().unwrap();
}
