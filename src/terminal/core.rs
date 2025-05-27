use crate::commands::CommandHandler;
use js_sys::Promise;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Document, Element};

pub struct Terminal {
    pub output_element: Element,
    pub command_processor: CommandHandler,
    pub prompt: String,
}

impl Terminal {
    pub fn new(document: &Document) -> Self {
        let output_element = document.get_element_by_id("terminal-output").unwrap();
        let command_processor = CommandHandler::new();
        let prompt = "objz@portfolio:~$ ".to_string();

        Self {
            output_element,
            command_processor,
            prompt,
        }
    }

    pub async fn start_intro(&self) {
        self.run_boot_sequence().await;
    }

    pub async fn start_shell(&self) {
        self.setup_input_system();
    }

    pub async fn sleep(&self, ms: i32) {
        let promise = Promise::new(&mut |resolve, _| {
            let w = window().unwrap();
            let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
        });
        let _ = JsFuture::from(promise).await;
    }
}
