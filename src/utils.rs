use js_sys::Promise;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Element, HtmlElement};

pub fn create_div(text: &str, class: Option<&str>) -> Element {
    let div = window()
        .unwrap()
        .document()
        .unwrap()
        .create_element("div")
        .unwrap();
    div.set_text_content(Some(text));

    if let Some(class) = class {
        div.set_class_name(class);
    }

    div
}

pub fn append_line(element: &Element, text: &str, class: Option<&str>) {
    let div = create_div(text, class);
    element.append_child(&div).unwrap();
}

pub fn append_prompt(element: &Element, prompt: &str) {
    let div = window()
        .unwrap()
        .document()
        .unwrap()
        .create_element("div")
        .unwrap();
    div.set_class_name("prompt-line");
    div.set_inner_html(&format!("<span class='prompt'>{}</span>", prompt));
    element.append_child(&div).unwrap();
}

pub fn scroll_to_bottom(element: &Element) {
    if let Ok(html_element) = element.clone().dyn_into::<HtmlElement>() {
        html_element.set_scroll_top(html_element.scroll_height());
    }
}

pub fn clear_output(element: &Element) {
    element.set_inner_html("");
}

pub async fn show_system_panic(element: &Element) {
    clear_output(element);

    let panic_lines = vec![
        "⚠️  CRITICAL SYSTEM ERROR ⚠️",
        "",
        "Deleting root filesystem...",
        "rm: removing /usr... ████████████░░░░ 75%",
        "rm: removing /var... ██████████████░░ 87%",
        "rm: removing /etc... ████████████████ 100%",
        "",
        "SYSTEM DESTROYED ☠️",
        "",
        "Just kidding! This is a portfolio website, not your actual system.",
        "Nice try though! 😉",
        "",
        "(Don't actually run 'sudo rm -rf /' on real systems!)",
        "",
    ];

    for line in panic_lines {
        let class = if line.contains("⚠️") || line.contains("☠️") {
            Some("error")
        } else if line.contains("████") {
            Some("warning")
        } else {
            None
        };
        append_line(element, line, class);

        let promise = Promise::new(&mut |resolve, _| {
            let window = window().unwrap();
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 300);
        });
        let _ = JsFuture::from(promise).await;
    }

    let promise = Promise::new(&mut |resolve, _| {
        let window = window().unwrap();
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 2000);
    });
    let _ = JsFuture::from(promise).await;

    clear_output(element);
    append_line(
        element,
        "System restored! Terminal is back online.",
        Some("success"),
    );
    append_line(element, "", None);
}
