mod flag;
mod tui;

use std::env;

fn main() {
    let arguments: Vec<String> = env::args().collect();

    match arguments.get(1).map(String::as_str) {
        None => tui::show_menu(),
        Some(unknown) => tui::handle_unknown(unknown),
    }
}
