//use mist_run_utils::run::Run;
use fltk::{app::*, window::*, table, draw, input};

static HEADERS: [&'static str; 3] = ["Split Name", "Personal Best", "Gold"];

fn main() {
	let app = App::default();
	let mut win = Window::default().with_size(500, 600).center_screen().with_label("mist split editor");
	let mut table = table::Table::new(5, 20, 490, 550, "qwerty");
	table.set_rows(10);
	table.set_row_header(true);
	table.set_cols(3);
	table.set_col_header(true);
	table.set_col_width(0, 200);
	table.end();
	win.make_resizable(false);
	win.end();
	win.show();
	table.draw_cell2(move |t, ctx, row, col, x, y, w, h| match ctx {
		table::TableContext::StartPage => {draw::set_font(Font::Courier, 14)}
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
            		inp.set_value("qwertuiop");
            		t.add(&inp);
            	}
    		_ => {}
    	});
	app.run().unwrap();
}
