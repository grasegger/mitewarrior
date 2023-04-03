use std::process::exit;

mod config;
mod mite;
mod model;
mod timew_input;
mod tw_entry;
mod ui;

use config::Config;
use mite::Mite;
use timew_input::TimewInput;
use ui::UI;

fn main() {
    let config = Config::load();
    let input = TimewInput::read_from_stdin();
    let mite = Mite::new(config.get_api_key(), config.mite_api_instance.clone());

    if let Some(mite) = mite {
        let mut ui = UI::new(input, mite.clone());
        ui.boot();

        let answers = ui.get_answers();

        for answer in answers {
            println!("{:?}", answer);
            mite.create_time_entry(answer);
        }
    } else {
        println!("The connection to mite was unsuccesful. Please check your config.");
        exit(1);
    }

    exit(0);
}
