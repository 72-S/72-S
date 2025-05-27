use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};

pub fn scroll_to_bottom(element: &Element) {
    let html_el: &HtmlElement = element.dyn_ref().unwrap();
    html_el.set_scroll_top(html_el.scroll_height());
}
