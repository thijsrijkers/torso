mod flag;
mod tui;
mod scaffold;
mod provisioner;
use std::env;

fn main() {
    let arguments: Vec<String> = env::args().collect();

    match arguments.get(1).map(String::as_str) {
        None => tui::show_menu(),
        Some("--init") => scaffold::loader::load_scaffold(),
        Some(unknown) => tui::handle_unknown(unknown),
    }
}
