use crate::terminal::Terminal;
use crate::utils::{append_line, clear_output, scroll_to_bottom, show_system_panic};
use std::collections::VecDeque;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, Event, HtmlInputElement, KeyboardEvent};

// Command history structure
#[derive(Clone)]
pub struct CommandHistory {
    commands: VecDeque<String>,
    current_index: Option<usize>,
    max_size: usize,
}

impl CommandHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            commands: VecDeque::new(),
            current_index: None,
            max_size,
        }
    }

    pub fn add_command(&mut self, command: String) {
        if !command.trim().is_empty() {
            // Remove duplicate if it exists
            if let Some(pos) = self.commands.iter().position(|x| x == &command) {
                self.commands.remove(pos);
            }

            // Add to front
            self.commands.push_front(command);

            // Maintain max size
            if self.commands.len() > self.max_size {
                self.commands.pop_back();
            }
        }
        self.current_index = None;
    }

    pub fn get_previous(&mut self) -> Option<String> {
        if self.commands.is_empty() {
            return None;
        }

        match self.current_index {
            None => {
                self.current_index = Some(0);
                self.commands.get(0).cloned()
            }
            Some(index) => {
                if index + 1 < self.commands.len() {
                    self.current_index = Some(index + 1);
                    self.commands.get(index + 1).cloned()
                } else {
                    self.commands.get(index).cloned()
                }
            }
        }
    }

    pub fn get_next(&mut self) -> Option<String> {
        match self.current_index {
            None => None,
            Some(0) => {
                self.current_index = None;
                Some(String::new())
            }
            Some(index) => {
                self.current_index = Some(index - 1);
                self.commands.get(index - 1).cloned()
            }
        }
    }
}

impl Terminal {
    pub fn setup_input_system(&self) {
        self.create_prompt_input_with_cursor();
        self.show_prompt();
        self.setup_enhanced_input_handler();
    }

    fn create_prompt_input_with_cursor(&self) {
        let document = window().unwrap().document().unwrap();
        let terminal_body = document.get_element_by_id("terminal-body").unwrap();

        let prompt_line_div = document.create_element("div").unwrap();
        prompt_line_div.set_class_name("prompt-line");

        let prompt_span = document.create_element("span").unwrap();
        prompt_span.set_class_name("prompt");
        prompt_span.set_text_content(Some(&self.prompt));

        // Create container for input and cursor
        let input_container = document.create_element("div").unwrap();
        input_container.set_class_name("input-container");

        let input_element = document.create_element("input").unwrap();
        let input_element = input_element.dyn_into::<HtmlInputElement>().unwrap();
        input_element.set_type("text");
        input_element.set_id("terminal-input");
        input_element.set_class_name("terminal-input-field");
        input_element.set_attribute("autocomplete", "off").unwrap();
        input_element.set_attribute("spellcheck", "false").unwrap();

        // Create animated cursor
        let cursor = document.create_element("span").unwrap();
        cursor.set_class_name("animated-cursor");
        cursor.set_id("input-cursor");
        cursor.set_text_content(Some("█"));

        input_container.append_child(&input_element).unwrap();
        input_container.append_child(&cursor).unwrap();

        prompt_line_div.append_child(&prompt_span).unwrap();
        prompt_line_div.append_child(&input_container).unwrap();
        terminal_body.append_child(&prompt_line_div).unwrap();

        // Setup cursor animation after creating elements
        self.setup_cursor_animation();
        self.setup_typing_animation();
    }

    fn setup_enhanced_input_handler(&self) {
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

        // Initialize command history with some default commands
        let mut command_history = CommandHistory::new(50);
        command_history.add_command("help".to_string());
        command_history.add_command("about".to_string());
        command_history.add_command("projects".to_string());
        command_history.add_command("skills".to_string());
        command_history.add_command("contact".to_string());
        command_history.add_command("clear".to_string());
        command_history.add_command("ls".to_string());
        command_history.add_command("whoami".to_string());

        let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            let key = event.key();

            match key.as_str() {
                "Enter" => {
                    let input_value = input_element_clone.value();

                    // Add to history if not empty
                    if !input_value.trim().is_empty() {
                        command_history.add_command(input_value.clone());
                    }

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
                "ArrowUp" => {
                    event.prevent_default();
                    if let Some(command) = command_history.get_previous() {
                        input_element_clone.set_value(&command);
                        // Move cursor to end
                        let _ = input_element_clone.set_selection_start(Some(command.len() as u32));
                        let _ = input_element_clone.set_selection_end(Some(command.len() as u32));
                    }
                }
                "ArrowDown" => {
                    event.prevent_default();
                    if let Some(command) = command_history.get_next() {
                        input_element_clone.set_value(&command);
                        // Move cursor to end
                        let _ = input_element_clone.set_selection_start(Some(command.len() as u32));
                        let _ = input_element_clone.set_selection_end(Some(command.len() as u32));
                    }
                }
                _ => {
                    // For any other key, trigger typing animation
                    // This will be handled by CSS animations
                }
            }
        }) as Box<dyn FnMut(_)>);

        input_element
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .unwrap();

        closure.forget();
    }

    fn setup_cursor_animation(&self) {
        let document = window().unwrap().document().unwrap();
        let input_element = document.get_element_by_id("terminal-input").unwrap();
        let cursor = document.get_element_by_id("input-cursor").unwrap();

        // Focus handler - using generic Event instead of FocusEvent
        let cursor_clone = cursor.clone();
        let focus_closure = Closure::wrap(Box::new(move |_event: Event| {
            cursor_clone.set_class_name("animated-cursor focused");
        }) as Box<dyn FnMut(_)>);

        input_element
            .add_event_listener_with_callback("focus", focus_closure.as_ref().unchecked_ref())
            .unwrap();
        focus_closure.forget();

        // Blur handler - using generic Event instead of FocusEvent
        let cursor_clone = cursor.clone();
        let blur_closure = Closure::wrap(Box::new(move |_event: Event| {
            cursor_clone.set_class_name("animated-cursor");
        }) as Box<dyn FnMut(_)>);

        input_element
            .add_event_listener_with_callback("blur", blur_closure.as_ref().unchecked_ref())
            .unwrap();
        blur_closure.forget();
    }

    fn setup_typing_animation(&self) {
        let document = window().unwrap().document().unwrap();
        let input_element = document.get_element_by_id("terminal-input").unwrap();
        let input_element_clone = input_element.clone();

        // Add typing effect on input - using generic Event instead of InputEvent
        let typing_closure = Closure::wrap(Box::new(move |_event: Event| {
            // Add temporary typing class for animation
            if let Some(input) = input_element_clone.dyn_ref::<web_sys::HtmlInputElement>() {
                input.set_class_name("terminal-input-field typing");

                // Remove typing class after animation
                let input_clone = input.clone();
                let timeout_closure = Closure::wrap(Box::new(move || {
                    input_clone.set_class_name("terminal-input-field");
                }) as Box<dyn FnMut()>);

                let _ = window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        timeout_closure.as_ref().unchecked_ref(),
                        200,
                    );
                timeout_closure.forget();
            }
        }) as Box<dyn FnMut(_)>);

        input_element
            .add_event_listener_with_callback("input", typing_closure.as_ref().unchecked_ref())
            .unwrap();
        typing_closure.forget();
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
