use crate::terminal::Terminal;
use crate::utils::{append_line, clear_output, scroll_to_bottom, show_system_panic};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, HtmlInputElement, KeyboardEvent};

impl Terminal {
    pub fn setup_input_system(&self) {
        self.create_prompt_input();
        self.show_prompt();
        self.setup_input_handler();
    }

    fn create_prompt_input(&self) {
        let document = window().unwrap().document().unwrap();
        let terminal_body = document.get_element_by_id("terminal-body").unwrap();

        let prompt_line_div = document.create_element("div").unwrap();
        prompt_line_div.set_class_name("prompt-line");

        let prompt_span = document.create_element("span").unwrap();
        prompt_span.set_class_name("prompt");
        prompt_span.set_text_content(Some(&self.prompt));

        let input_element = document.create_element("input").unwrap();
        let input_element = input_element.dyn_into::<HtmlInputElement>().unwrap();
        input_element.set_type("text");
        input_element.set_id("terminal-input");
        input_element.set_attribute("autocomplete", "off").unwrap();
        input_element.set_attribute("spellcheck", "false").unwrap();

        prompt_line_div.append_child(&prompt_span).unwrap();
        prompt_line_div.append_child(&input_element).unwrap();
        terminal_body.append_child(&prompt_line_div).unwrap();
    }

    fn setup_input_handler(&self) {
        let output_element = self.output_element.clone();
        let document = window().unwrap().document().unwrap();
        let input_element = document
            .get_element_by_id("terminal-input")
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();

        let input_element_clone = input_element.clone();
        let mut command_processor = self.command_processor.clone();
        let prompt = self.prompt.clone();

        let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            if event.key() == "Enter" {
                let input_value = input_element_clone.value();
                input_element_clone.set_value("");

                let command_line = format!("{}{}", prompt, input_value);
                append_line(&output_element, &command_line, Some("command"));

                let output = command_processor.process_command(&input_value);

                if output == "CLEAR_SCREEN" {
                    clear_output(&output_element);
                } else if output == "SYSTEM_PANIC" {
                    spawn_local({
                        let output_element = output_element.clone();
                        async move {
                            show_system_panic(&output_element).await;
                        }
                    });
                } else {
                    for line in output.lines() {
                        let class = if line.starts_with("ERROR") || line.contains("not found") {
                            Some("error")
                        } else if line.contains("[OK]") || line.contains("SUCCESS") {
                            Some("success")
                        } else if line.starts_with("NAVIGATION:")
                            || line.starts_with("SYSTEM:")
                            || line.starts_with("PORTFOLIO:")
                            || line.starts_with("NETWORK:")
                            || line.starts_with("EASTER EGGS:")
                        {
                            Some("header")
                        } else if line.starts_with("  ") && line.contains(" - ") {
                            Some("info")
                        } else if line.contains("Available commands:")
                            || line.contains("Type any command")
                        {
                            Some("subheader")
                        } else if line.contains("GitHub:") || line.contains("http") {
                            Some("link")
                        } else if line.contains("█") || line.contains("╔") || line.contains("┌")
                        {
                            Some("ascii")
                        } else if line.contains("/")
                            && (line.contains("ls") || line.contains("cat"))
                        {
                            Some("directory")
                        } else if line.is_empty() {
                            None
                        } else {
                            Some("file")
                        };
                        append_line(&output_element, line, class);
                    }
                }

                scroll_to_bottom(&output_element);
            }
        }) as Box<dyn FnMut(_)>);

        input_element
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .unwrap();

        closure.forget();
    }

    fn show_prompt(&self) {
        let document = window().unwrap().document().unwrap();
        let input_element = document
            .get_element_by_id("terminal-input")
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();
        input_element.focus().unwrap();
    }
}
