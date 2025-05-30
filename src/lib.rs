use commands::system;
use input::setup::InputSetup;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, HtmlInputElement};

mod ascii_art;
mod boot;
mod commands;
mod input;
mod terminal;
mod utils;

use terminal::Terminal;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    // Initialize system time
    commands::system::init();

    // Create terminal with new line buffer system
    let terminal = Terminal::new(&document);

    // Get hidden input element
    let hidden_input = document
        .get_element_by_id("hidden-input")
        .expect("hidden input not found")
        .dyn_into::<HtmlInputElement>()
        .expect("element is not an input");

    // Set up input handling with new system
    InputSetup::setup(&terminal, &hidden_input);

    // Initialize boot sequence
    wasm_bindgen_futures::spawn_local(async move {
        terminal.init_boot().await;
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
