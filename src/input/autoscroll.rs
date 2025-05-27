use web_sys::window;

pub fn ensure_autoscroll() {
    if let Some(window) = window() {
        if let Some(doc) = window.document() {
            if let Some(body) = doc.get_element_by_id("terminal-body") {
                body.set_scroll_top(body.scroll_height());
            }
        }
    }
}
