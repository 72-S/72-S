use crate::commands::CommandHandler;
use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Document, Element};

#[derive(Clone)]
pub struct Terminal {
    pub canvas: Element,
    pub command_handler: CommandHandler,
    pub base_prompt: String,
    pub height: i32,
}

impl Terminal {
    pub fn new(document: &Document) -> Self {
        let canvas = document
            .get_element_by_id("terminal")
            .expect("canvas not found");
        let command_handler = CommandHandler::new();
        let base_prompt = "anonym@objz".to_string();
        let height = 760;

        Self {
            canvas,
            command_handler,
            base_prompt,
            height,
        }
    }

    pub fn get_current_prompt(&self) -> String {
        let cwd = self.command_handler.get_current_directory();
        let display_path = if cwd == "/home/objz" {
            "~".to_string()
        } else if cwd.starts_with("/home/objz/") {
            format!("~{}", &cwd["/home/objz".len()..])
        } else {
            cwd
        };

        format!("{}:{}$ ", self.base_prompt, display_path)
    }

    pub async fn sleep(&self, ms: i32) {
        let promise = Promise::new(&mut |resolve, _reject| {
            let window = window().unwrap();
            let closure = Closure::once_into_js(move || {
                resolve.call0(&JsValue::UNDEFINED).unwrap();
            });
            window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    ms,
                )
                .unwrap();
        });

        let _ = JsFuture::from(promise).await;
    }
}
