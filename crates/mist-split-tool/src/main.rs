use fltk::{app, draw, input, table, window::*};
use mist_run_utils::run::Run;
use std::env::current_dir;
use tinyfiledialogs as tfd;
use std::sync::Mutex;
use std::convert::TryInto;

use lazy_static::lazy_static;

static HEADERS: [&'static str; 3] = ["Split Name", "Personal Best", "Gold"];

lazy_static!{
	static ref RUN: Mutex<Run> = Mutex::new(Run::default());
}

fn open_split_file() -> Option<String> {
    let cwd = current_dir().unwrap();
    let mut dir = cwd.to_string_lossy();
    dir.to_mut().push('/');
    return tfd::open_file_dialog("Open a split file (msf or lss)", &dir, Some((&["*.msf", "*.lss"], "")));
}

fn main() {
    let path = open_split_file();
    match path {
	Some(p) => {
    		*RUN.lock().unwrap() = Run::from_msf_file(&p).unwrap();
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
    let mut table = table::Table::new(5, 60, 503, 550, "qwerty");
    table.set_rows(RUN.lock().unwrap().split_names().len().try_into().unwrap());
    table.set_row_header(true);
    table.set_cols(3);
    table.set_col_header(true);
    table.set_col_width(0, 180);
    table.set_col_width(1, 140);
    table.set_col_width(2, 140);
    table.end();
    win.make_resizable(false);
    win.end();
    win.show();
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
                let inp = input::Input::new(x, y, w, h, "");
                if col == 0 && (row as usize) < RUN.lock().unwrap().split_names().len() {
                    	inp.set_value(&RUN.lock().unwrap().split_names()[row as usize]);
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
