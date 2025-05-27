use crate::terminal::Terminal;
use crate::utils::{append_line, clear_output, scroll_to_bottom, show_system_panic};
use std::collections::VecDeque;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, Event, HtmlTextAreaElement, KeyboardEvent};

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

// Function to ensure autoscroll to bottom
fn ensure_autoscroll() {
    if let Some(window) = window() {
        if let Some(document) = window.document() {
            if let Some(terminal_body) = document.get_element_by_id("terminal-body") {
                let scroll_height = terminal_body.scroll_height();
                terminal_body.set_scroll_top(scroll_height);
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

        // Create container for input
        let input_container = document.create_element("div").unwrap();
        input_container.set_class_name("input-container");

        // Use textarea instead of input for better text wrapping
        let input_element = document.create_element("textarea").unwrap();
        let input_element = input_element.dyn_into::<HtmlTextAreaElement>().unwrap();
        input_element.set_id("terminal-input");
        input_element.set_class_name("terminal-input-field");
        input_element.set_attribute("autocomplete", "off").unwrap();
        input_element.set_attribute("spellcheck", "false").unwrap();
        input_element.set_attribute("rows", "1").unwrap();
        input_element.set_attribute("wrap", "soft").unwrap();

        input_container.append_child(&input_element).unwrap();

        prompt_line_div.append_child(&prompt_span).unwrap();
        prompt_line_div.append_child(&input_container).unwrap();
        terminal_body.append_child(&prompt_line_div).unwrap();

        // Setup auto-resize and typing animation
        self.setup_auto_resize();
        self.setup_typing_animation();

        // Ensure initial scroll to bottom
        ensure_autoscroll();
    }

    fn setup_auto_resize(&self) {
        let document = window().unwrap().document().unwrap();
        let input_element = document.get_element_by_id("terminal-input").unwrap();
        let input_element_clone = input_element.clone();

        let resize_closure = Closure::wrap(Box::new(move |_event: Event| {
            if let Some(textarea) = input_element_clone.dyn_ref::<web_sys::HtmlTextAreaElement>() {
                // Reset height to auto to get the correct scroll height
                textarea.style().set_property("height", "auto").unwrap();

                // Set height based on scroll height with min/max constraints
                let scroll_height = textarea.scroll_height();
                let line_height = 22;
                let min_height = line_height;
                let max_height = line_height * 10;

                let new_height = scroll_height.max(min_height).min(max_height);
                textarea
                    .style()
                    .set_property("height", &format!("{}px", new_height))
                    .unwrap();

                // Update rows attribute
                let rows = (new_height / line_height).max(1);
                textarea.set_attribute("rows", &rows.to_string()).unwrap();

                // Ensure autoscroll after resize
                ensure_autoscroll();
            }
        }) as Box<dyn FnMut(_)>);

        input_element
            .add_event_listener_with_callback("input", resize_closure.as_ref().unchecked_ref())
            .unwrap();
        resize_closure.forget();
    }

    fn setup_enhanced_input_handler(&self) {
        let output_element = self.output_element.clone();
        let document = window().unwrap().document().unwrap();
        let input_element = document
            .get_element_by_id("terminal-input")
            .unwrap()
            .dyn_into::<HtmlTextAreaElement>()
            .unwrap();

        let input_element_clone = input_element.clone();
        let mut command_processor = self.command_processor.clone();
        let prompt = self.prompt.clone();

        // Initialize command history
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
                    if !event.shift_key() {
                        event.prevent_default();

                        let input_value = input_element_clone.value();

                        if !input_value.trim().is_empty() {
                            command_history.add_command(input_value.clone());
                        }

                        input_element_clone.set_value("");

                        // Reset textarea height
                        input_element_clone
                            .style()
                            .set_property("height", "auto")
                            .unwrap();
                        input_element_clone.set_attribute("rows", "1").unwrap();

                        let command_line = format!("{}{}", prompt, input_value);
                        append_line(&output_element, &command_line, Some("command"));

                        // Autoscroll after adding command
                        ensure_autoscroll();

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
                            // Process output lines with improved animations
                            for line in output.lines() {
                                let class =
                                    if line.starts_with("ERROR") || line.contains("not found") {
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
                                    } else if line.contains("█")
                                        || line.contains("╔")
                                        || line.contains("┌")
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

                                // Autoscroll after each line for smooth scrolling effect
                                ensure_autoscroll();
                            }
                        }

                        // Final autoscroll to ensure we're at the bottom
                        let final_scroll_closure = Closure::wrap(Box::new(move || {
                            ensure_autoscroll();
                        })
                            as Box<dyn FnMut()>);

                        let _ = window()
                            .unwrap()
                            .set_timeout_with_callback_and_timeout_and_arguments_0(
                                final_scroll_closure.as_ref().unchecked_ref(),
                                50,
                            );
                        final_scroll_closure.forget();
                    }
                }
                "ArrowUp" => {
                    let cursor_pos = input_element_clone
                        .selection_start()
                        .unwrap_or(None)
                        .unwrap_or(0);
                    if cursor_pos == 0 {
                        event.prevent_default();
                        if let Some(command) = command_history.get_previous() {
                            input_element_clone.set_value(&command);
                            let len = command.len() as u32;
                            let _ = input_element_clone.set_selection_start(Some(len));
                            let _ = input_element_clone.set_selection_end(Some(len));

                            // Trigger resize
                            let resize_event = web_sys::Event::new("input").unwrap();
                            let _ = input_element_clone.dispatch_event(&resize_event);
                        }
                    }
                }
                "ArrowDown" => {
                    let text_len = input_element_clone.value().len() as u32;
                    let cursor_pos = input_element_clone
                        .selection_start()
                        .unwrap_or(None)
                        .unwrap_or(0);
                    if cursor_pos == text_len {
                        event.prevent_default();
                        if let Some(command) = command_history.get_next() {
                            input_element_clone.set_value(&command);
                            let len = command.len() as u32;
                            let _ = input_element_clone.set_selection_start(Some(len));
                            let _ = input_element_clone.set_selection_end(Some(len));

                            // Trigger resize
                            let resize_event = web_sys::Event::new("input").unwrap();
                            let _ = input_element_clone.dispatch_event(&resize_event);
                        }
                    }
                }
                _ => {
                    // Other keys handled by typing animation
                }
            }
        }) as Box<dyn FnMut(_)>);

        input_element
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .unwrap();

        closure.forget();
    }

    fn setup_typing_animation(&self) {
        let document = window().unwrap().document().unwrap();
        let input_element = document.get_element_by_id("terminal-input").unwrap();
        let input_element_clone = input_element.clone();

        // Enhanced typing animation
        let typing_closure = Closure::wrap(Box::new(move |_event: Event| {
            if let Some(textarea) = input_element_clone.dyn_ref::<web_sys::HtmlTextAreaElement>() {
                // Add typing class
                textarea.set_class_name("terminal-input-field typing");

                // Remove typing class after animation
                let textarea_clone = textarea.clone();
                let timeout_closure = Closure::wrap(Box::new(move || {
                    textarea_clone.set_class_name("terminal-input-field");
                }) as Box<dyn FnMut()>);

                let _ = window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        timeout_closure.as_ref().unchecked_ref(),
                        150, // Reduced timeout for more responsive animation
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
            .dyn_into::<HtmlTextAreaElement>()
            .unwrap();
        input_element.focus().unwrap();

        // Ensure autoscroll when showing prompt
        ensure_autoscroll();
    }
}
