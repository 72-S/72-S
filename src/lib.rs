use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;

mod ascii_art;
mod boot;
mod commands;
mod input;
mod terminal;
mod utils;

use terminal::Terminal;

#[wasm_bindgen(start)]
pub fn main() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    let window = window().unwrap();
    let document = window.document().unwrap();

    let term = Terminal::new(&document);
    spawn_local(async move {
        term.start_intro().await;
        term.start_shell().await;
    });
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    log("Terminal loaded");
}
