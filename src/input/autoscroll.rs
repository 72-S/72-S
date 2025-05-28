use wasm_bindgen::prelude::*;
use web_sys::window;

pub fn ensure_autoscroll() {
    if let Some(window) = window() {
        if let Some(doc) = window.document() {
            if let Some(body) = doc.get_element_by_id("terminal-body") {
                // Force immediate scroll
                body.set_scroll_top(body.scroll_height());

                // Also trigger the JavaScript version
                let _ =
                    js_sys::eval("if (window.objzEnsureAutoscroll) window.objzEnsureAutoscroll();");

                // Force a small delay and scroll again to ensure it works
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
