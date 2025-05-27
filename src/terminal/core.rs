use crate::commands::CommandHandler;
use js_sys::Promise;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Document, Element};

pub struct Terminal {
    pub output_element: Element,
    pub command_processor: CommandHandler,
    pub base_prompt: String,
}

impl Terminal {
    pub fn new(document: &Document) -> Self {
        let output_element = document.get_element_by_id("terminal-output").unwrap();
        let command_processor = CommandHandler::new();
        let base_prompt = "anonym@objz".to_string();

        Self {
            output_element,
            command_processor,
            base_prompt,
        }
    }

    pub fn get_current_prompt(&self) -> String {
        let cwd = self.command_processor.get_current_directory();
        let display_path = if cwd == "/home/objz" {
            "~".to_string()
        } else if cwd.starts_with("/home/objz/") {
            format!("~{}", &cwd["/home/objz".len()..])
        } else {
            cwd
        };

        format!("{}:{}$ ", self.base_prompt, display_path)
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
