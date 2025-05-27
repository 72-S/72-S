use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;

mod animator;
mod ascii_art;
mod commands;
mod terminal;

use terminal::Terminal;

#[wasm_bindgen(start)]
pub fn main() {
    // Use set_once() from the console_error_panic_hook crate
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    let window = window().unwrap();
    let document = window.document().unwrap();

    // Initialize terminal
    let terminal = Terminal::new(&document);

    // Start the hacking intro animation
    spawn_local(async move {
        terminal.start_hacking_intro().await;
        terminal.start_interactive_shell().await;
    });
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    log("Welcome to objz's Advanced Terminal Portfolio!");
}
