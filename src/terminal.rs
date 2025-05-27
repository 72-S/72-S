use crate::commands::CommandProcessor;
use crate::utils::{append_line, scroll_to_bottom};
use js_sys::Promise;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Document, Element};

pub struct Terminal {
    pub output_element: Element,
    pub command_processor: CommandProcessor,
    pub prompt: String,
}

impl Terminal {
    pub fn new(document: &Document) -> Self {
        let output_element = document.get_element_by_id("terminal-output").unwrap();
        let command_processor = CommandProcessor::new();
        let prompt = "objz@portfolio:~$ ".to_string();

        Self {
            output_element,
            command_processor,
            prompt,
        }
    }

    pub async fn start_hacking_intro(&self) {
        self.run_boot_sequence().await;
    }

    pub async fn start_interactive_shell(&self) {
        self.setup_input_system();
    }

    // Faster boot animation with improved spinner (removed unused color parameter)
    pub async fn add_line_with_boot_animation(&self, task: &str, status: &str, _color: &str) {
        let div = self.create_div_element("", Some("boot-line"));
        self.output_element.append_child(&div).unwrap();
        self.scroll_to_bottom();

        self.sleep(20).await; // Reduced from 50ms

        let spinners = vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let mut spinner_cycle = 0;

        // Fewer spinner cycles for faster animation
        for _ in 0..4 {
            let spinner_text = format!("{} {}", task, spinners[spinner_cycle % spinners.len()]);
            div.set_inner_html(&spinner_text);
            self.scroll_to_bottom();
            spinner_cycle += 1;
            self.sleep(60).await; // Reduced from 100ms
        }

        if !status.is_empty() {
            // Create HTML with separate styling for task and status
            let final_html = format!("{} <span class=\"status\">{}</span>", task, status);
            div.set_inner_html(&final_html);
        } else {
            div.set_inner_html(task);
        }

        self.scroll_to_bottom();
    }

    // New method for smooth typing animation
    pub async fn add_line_with_typing(&self, text: &str, typing_speed: u32) {
        let div = self.create_div_element("", None);
        div.set_class_name("typing-line");
        self.output_element.append_child(&div).unwrap();
        self.scroll_to_bottom();

        let mut current_text = String::new();
        let chars: Vec<char> = text.chars().collect();

        for (i, ch) in chars.iter().enumerate() {
            current_text.push(*ch);

            // Add cursor at the end while typing
            let display_text = if i < chars.len() - 1 {
                format!("{}<span class=\"typing-cursor\">█</span>", current_text)
            } else {
                current_text.clone()
            };

            div.set_inner_html(&display_text);
            self.scroll_to_bottom();
            self.sleep(typing_speed as i32).await;
        }
    }

    // Typing with color support
    pub async fn add_line_with_typing_color(&self, text: &str, typing_speed: u32, color: &str) {
        let div = self.create_div_element("", Some(color));
        div.set_class_name(&format!("typing-line {}", color));
        self.output_element.append_child(&div).unwrap();
        self.scroll_to_bottom();

        let mut current_text = String::new();
        let chars: Vec<char> = text.chars().collect();

        for (i, ch) in chars.iter().enumerate() {
            current_text.push(*ch);

            let display_text = if i < chars.len() - 1 {
                format!("{}<span class=\"typing-cursor\">█</span>", current_text)
            } else {
                current_text.clone()
            };

            div.set_inner_html(&display_text);
            self.scroll_to_bottom();
            self.sleep(typing_speed as i32).await;
        }
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
