use wasm_bindgen::prelude::*;
use web_sys::window;

pub fn ensure_autoscroll() {
    if let Some(window) = window() {
        if let Some(doc) = window.document() {
            if let Some(body) = doc.get_element_by_id("terminal-body") {
                body.set_scroll_top(body.scroll_height());

                let _ =
                    js_sys::eval("if (window.objzEnsureAutoscroll) window.objzEnsureAutoscroll();");

                let body_clone = body.clone();
                let closure = Closure::once_into_js(Box::new(move || {
                    body_clone.set_scroll_top(body_clone.scroll_height());
                }) as Box<dyn FnOnce()>);

                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    0,
                );
            }
        }
    }
}

pub fn trim_output(max_height: i32) {
    if let Some(doc) = window().unwrap().document() {
        if let Some(output) = doc.get_element_by_id("terminal-output") {
            let output_el: web_sys::HtmlElement = output.unchecked_into();
            while output_el.scroll_height() > max_height && output_el.child_element_count() > 1 {
                let first = output_el.first_element_child();
                if let Some(f) = first {
                    let _ = output_el.remove_child(&f);
                }
            }
        }
    }
}
