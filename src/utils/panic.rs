use crate::utils::dom::{append_line, clear_output};
use js_sys::Promise;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Element};

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
