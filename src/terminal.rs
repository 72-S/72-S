use crate::animator::Animator;
use crate::commands::CommandProcessor;
use crate::utils::{append_line, scroll_to_bottom};
use js_sys::Promise;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Document, Element};

pub struct Terminal {
    pub output_element: Element,
    pub command_processor: CommandProcessor,
    pub prompt: String,
    pub animator: Animator,
}

impl Terminal {
    pub fn new(document: &Document) -> Self {
        let output_element = document.get_element_by_id("terminal-output").unwrap();
        let command_processor = CommandProcessor::new();
        let prompt = "objz@portfolio:~$ ".to_string();
        let animator = Animator::new();

        Self {
            output_element,
            command_processor,
            prompt,
            animator,
        }
    }

    pub async fn start_hacking_intro(&self) {
        self.run_boot_sequence().await;
    }

    pub async fn start_interactive_shell(&self) {
        self.setup_input_system();
    }

    // Animation methods
    pub async fn add_line_with_boot_animation(&self, task: &str, status: &str, color: &str) {
        let div = self.create_div_element(task, None);
        self.output_element.append_child(&div).unwrap();
        self.scroll_to_bottom();

        self.sleep(50).await;

        let spinners = vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let mut spinner_cycle = 0;

        for _ in 0..8 {
            let spinner_text = format!("{} {}", task, spinners[spinner_cycle % spinners.len()]);
            div.set_text_content(Some(&spinner_text));
            self.scroll_to_bottom();
            spinner_cycle += 1;
            self.sleep(100).await;
        }

        if !status.is_empty() {
            let final_text = format!("{} {}", task, status);
            div.set_text_content(Some(&final_text));
        } else {
            div.set_text_content(Some(task));
        }

        match color {
            "green" => div.set_class_name("success"),
            "yellow" => div.set_class_name("warning"),
            "red" => div.set_class_name("error"),
            _ => div.set_class_name(""),
        }

        self.scroll_to_bottom();
    }

    pub async fn add_line_instant(&self, text: &str) {
        self.append_line(text, None);
        self.scroll_to_bottom();
    }

    pub async fn add_line_instant_with_color(&self, text: &str, color: &str) {
        self.append_line(text, Some(color));
        self.scroll_to_bottom();
    }

    pub fn create_div_element(&self, text: &str, class: Option<&str>) -> Element {
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

    pub async fn add_line(&self, text: &str) {
        self.append_line(text, None);
        self.scroll_to_bottom();
    }

    pub fn append_line(&self, text: &str, class: Option<&str>) {
        append_line(&self.output_element, text, class);
    }

    pub fn clear_output(&self) {
        self.output_element.set_inner_html("");
    }

    pub fn scroll_to_bottom(&self) {
        scroll_to_bottom(&self.output_element);
    }

    pub async fn sleep(&self, ms: i32) {
        let promise = Promise::new(&mut |resolve, _| {
            let window = window().unwrap();
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
        });
        let _ = JsFuture::from(promise).await;
    }
}
