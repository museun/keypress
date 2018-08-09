extern crate env_logger;
// #[macro_use]
// extern crate log;

extern crate keypress;
use keypress::*;

fn main() {
    env_logger::init();

    let config = load_or_create_config();

    let tx = websocket::run_in_thread(&config.host);
    let tx = tx.clone();
    Hook::run(config, move |event| {
        if event.state == KeyState::Pressed {
            tx.send(event.into());
        }
    });
}
