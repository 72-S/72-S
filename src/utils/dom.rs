use web_sys::{window, Element};

use crate::input::autoscroll::{ensure_autoscroll, trim_output};

pub fn create_div(text: &str, class: Option<&str>) -> Element {
    let div = window()
        .unwrap()
        .document()
        .unwrap()
        .create_element("div")
        .unwrap();

    if text.is_empty() {
        div.set_inner_html("");
    } else {
        div.set_text_content(Some(text));
    }

    if let Some(class) = class {
        div.set_class_name(class);
    }

    div
}

pub fn append_line(element: &Element, text: &str, class: Option<&str>) {
    let div = create_div(text, class);
    element.append_child(&div).unwrap();
    trim_output(760);
    ensure_autoscroll();
}

pub fn clear_output(element: &Element) {
    element.set_inner_html("");
}
