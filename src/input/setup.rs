use crate::commands::CommandHandler;
use crate::terminal::{
    autocomplete::{find_common_prefix, AutoComplete, CompletionResult},
    line_buffer, InputMode, LineType, Terminal,
};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, HtmlInputElement, KeyboardEvent};

thread_local! {
    static CURRENT_INPUT: RefCell<String> = RefCell::new(String::new());
    static IS_FOCUSED: RefCell<bool> = RefCell::new(true);
    static AUTOCOMPLETE: RefCell<AutoComplete> = RefCell::new(AutoComplete::new());
}

pub struct CommandHistory {
    history: Vec<String>,
    current_index: Option<usize>,
}

impl CommandHistory {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            current_index: None,
        }
    }

    pub fn add(&mut self, command: String) {
        if !command.trim().is_empty() && self.history.last() != Some(&command) {
            self.history.push(command);
        }
        self.current_index = None;
    }

    pub fn previous(&mut self) -> Option<&String> {
        if self.history.is_empty() {
            return None;
        }

        let new_index = match self.current_index {
            None => self.history.len() - 1,
            Some(0) => return self.history.get(0),
            Some(i) => i - 1,
        };

        self.current_index = Some(new_index);
        self.history.get(new_index)
    }

    pub fn next(&mut self) -> Option<&String> {
        match self.current_index {
            None => None,
            Some(i) if i >= self.history.len() - 1 => {
                self.current_index = None;
                None
            }
            Some(i) => {
                let new_index = i + 1;
                self.current_index = Some(new_index);
                self.history.get(new_index)
            }
        }
    }
}

pub struct InputSetup;

impl InputSetup {
    pub fn setup(terminal: &Terminal, hidden_input: &HtmlInputElement) {
        let history = CommandHistory::new();
        let processor = terminal.command_handler.clone();

        let terminal_clone = terminal.clone();
        let hidden_input_clone = hidden_input.clone();

        // Set up input event listener for real-time updates
        let input_callback = {
            let terminal = terminal_clone.clone();
            let hidden_input = hidden_input_clone.clone();

            Closure::wrap(Box::new(move |_event: web_sys::Event| {
                let current_value = hidden_input.value();
                CURRENT_INPUT.with(|input| {
                    *input.borrow_mut() = current_value.clone();
                });

                // Update line buffer state and render
                let cursor_pos = hidden_input
                    .selection_start()
                    .unwrap_or(Some(0))
                    .unwrap_or(0) as usize;

                line_buffer::update_input_state(current_value, cursor_pos);
                terminal.render();
            }) as Box<dyn FnMut(_)>)
        };

        hidden_input
            .add_event_listener_with_callback("input", input_callback.as_ref().unchecked_ref())
            .unwrap();
        input_callback.forget();

        // Set up keydown event listener
        let keydown_callback = {
            let terminal = terminal_clone.clone();
            let hidden_input = hidden_input_clone.clone();
            let history = RefCell::new(history);
            let processor = RefCell::new(processor);

            Closure::wrap(Box::new(move |event: KeyboardEvent| {
                let current_input = CURRENT_INPUT.with(|input| input.borrow().clone());

                match event.key().as_str() {
                    "Enter" => {
                        event.prevent_default();
                        Self::handle_enter(
                            &current_input,
                            &mut history.borrow_mut(),
                            &mut processor.borrow_mut(),
                            &terminal,
                            &hidden_input,
                        );
                    }
                    "ArrowUp" => {
                        event.prevent_default();
                        if let Some(cmd) = history.borrow_mut().previous() {
                            hidden_input.set_value(cmd);
                            CURRENT_INPUT.with(|input| {
                                *input.borrow_mut() = cmd.clone();
                            });
                            line_buffer::update_input_state(cmd.clone(), cmd.len());
                            terminal.render();
                        }
                    }
                    "ArrowDown" => {
                        event.prevent_default();
                        if let Some(cmd) = history.borrow_mut().next() {
                            hidden_input.set_value(cmd);
                            CURRENT_INPUT.with(|input| {
                                *input.borrow_mut() = cmd.clone();
                            });
                            line_buffer::update_input_state(cmd.clone(), cmd.len());
                        } else {
                            hidden_input.set_value("");
                            CURRENT_INPUT.with(|input| {
                                input.borrow_mut().clear();
                            });
                            line_buffer::update_input_state(String::new(), 0);
                        }
                        terminal.render();
                    }
                    "Tab" => {
                        event.prevent_default();
                        Self::handle_tab_completion(&terminal, &hidden_input, &current_input);
                    }
                    _ => {}
                }
            }) as Box<dyn FnMut(_)>)
        };

        hidden_input
            .add_event_listener_with_callback("keydown", keydown_callback.as_ref().unchecked_ref())
            .unwrap();
        keydown_callback.forget();

        // Set up focus/blur event listeners for the hidden input
        Self::setup_focus_blur_listeners(&terminal_clone, &hidden_input_clone);

        // Set up cursor blinking
        Self::setup_cursor_blink(&terminal_clone);

        // Set up custom focus/blur event listeners from JavaScript
        Self::setup_custom_focus_blur_listeners(&terminal_clone);

        // Initialize terminal
        terminal.prepare_for_input();
        let _ = hidden_input.focus();
    }

    fn setup_focus_blur_listeners(terminal: &Terminal, hidden_input: &HtmlInputElement) {
        let terminal_clone = terminal.clone();

        // Focus event listener
        let focus_callback = {
            let terminal = terminal_clone.clone();
            Closure::wrap(Box::new(move |_event: web_sys::Event| {
                IS_FOCUSED.with(|focused| {
                    *focused.borrow_mut() = true;
                });
                terminal.renderer.show_cursor();
                terminal.render();
            }) as Box<dyn FnMut(_)>)
        };

        hidden_input
            .add_event_listener_with_callback("focus", focus_callback.as_ref().unchecked_ref())
            .unwrap();
        focus_callback.forget();

        // Blur event listener
        let blur_callback = {
            let terminal = terminal_clone.clone();
            Closure::wrap(Box::new(move |_event: web_sys::Event| {
                IS_FOCUSED.with(|focused| {
                    *focused.borrow_mut() = false;
                });
                terminal.renderer.hide_cursor();
                terminal.render();
            }) as Box<dyn FnMut(_)>)
        };

        hidden_input
            .add_event_listener_with_callback("blur", blur_callback.as_ref().unchecked_ref())
            .unwrap();
        blur_callback.forget();
    }

    fn setup_custom_focus_blur_listeners(terminal: &Terminal) {
        let terminal_clone = terminal.clone();
        let window = window().unwrap();

        // Listen for custom terminalFocus event from JavaScript
        let focus_event_callback = {
            let terminal = terminal_clone.clone();
            Closure::wrap(Box::new(move |_event: web_sys::Event| {
                IS_FOCUSED.with(|focused| {
                    *focused.borrow_mut() = true;
                });
                terminal.renderer.show_cursor();
                terminal.render();
            }) as Box<dyn FnMut(_)>)
        };

        window
            .add_event_listener_with_callback(
                "terminalFocus",
                focus_event_callback.as_ref().unchecked_ref(),
            )
            .unwrap();
        focus_event_callback.forget();

        // Listen for custom terminalBlur event from JavaScript
        let blur_event_callback = {
            let terminal = terminal_clone.clone();
            Closure::wrap(Box::new(move |_event: web_sys::Event| {
                IS_FOCUSED.with(|focused| {
                    *focused.borrow_mut() = false;
                });
                terminal.renderer.hide_cursor();
                terminal.render();
            }) as Box<dyn FnMut(_)>)
        };

        window
            .add_event_listener_with_callback(
                "terminalBlur",
                blur_event_callback.as_ref().unchecked_ref(),
            )
            .unwrap();
        blur_event_callback.forget();
    }

    fn handle_enter(
        current_input: &str,
        history: &mut CommandHistory,
        processor: &mut CommandHandler,
        terminal: &Terminal,
        hidden_input: &HtmlInputElement,
    ) {
        let trimmed_input = current_input.trim();

        // Add command to history and buffer
        if !trimmed_input.is_empty() {
            history.add(trimmed_input.to_string());

            // Get current prompt and add command to buffer
            let prompt = terminal.get_current_prompt();
            line_buffer::add_command_line(&prompt, trimmed_input);
        }

        // Clear input
        hidden_input.set_value("");
        CURRENT_INPUT.with(|input| input.borrow_mut().clear());
        line_buffer::update_input_state(String::new(), 0);
        line_buffer::set_input_mode(InputMode::Processing);

        // Process command
        if !trimmed_input.is_empty() {
            let (result, _directory_changed) = processor.handle(trimmed_input);

            match result.as_str() {
                "CLEAR_SCREEN" => {
                    line_buffer::clear_buffer();
                    Self::prepare_for_next_input(terminal, hidden_input);
                }
                "SYSTEM_PANIC" => {
                    let terminal_clone = terminal.clone();
                    let hidden_input_clone = hidden_input.clone();
                    spawn_local(async move {
                        Self::handle_system_panic(&terminal_clone).await;
                        Self::prepare_for_next_input(&terminal_clone, &hidden_input_clone);
                    });
                }
                _ => {
                    // Add command output
                    if !result.is_empty() {
                        line_buffer::add_output_lines(&result, None);
                    }
                    Self::prepare_for_next_input(terminal, hidden_input);
                }
            }
        } else {
            Self::prepare_for_next_input(terminal, hidden_input);
        }
    }

    fn prepare_for_next_input(terminal: &Terminal, hidden_input: &HtmlInputElement) {
        // Update prompt with current directory
        let prompt = terminal.get_current_prompt();
        line_buffer::set_current_prompt(prompt);
        line_buffer::set_input_mode(InputMode::Normal);
        line_buffer::auto_scroll_to_bottom();

        // Render and focus
        terminal.render();
        let _ = hidden_input.focus();
    }

    fn handle_tab_completion(
        terminal: &Terminal,
        hidden_input: &HtmlInputElement,
        current_input: &str,
    ) {
        // Get current directory path from the command handler
        let current_path = {
            use crate::commands::filesystem::CURRENT_PATH;
            CURRENT_PATH.lock().unwrap().clone()
        };

        // Parse the input to identify command and arguments
        let trimmed = current_input.trim();
        let parts: Vec<&str> = trimmed.split_whitespace().collect();

        // Determine what we're completing
        let (command_prefix, completion_target) = if parts.is_empty() {
            // Empty input - complete commands
            ("", current_input)
        } else if parts.len() == 1 && !trimmed.ends_with(' ') {
            // Single word without trailing space - complete command
            ("", current_input)
        } else {
            // Multiple words or single word with trailing space - complete path/argument
            if parts.len() == 1 {
                // Single command with trailing space
                (trimmed, "")
            } else {
                // Command with arguments - complete the last argument
                let last_space_idx = current_input.rfind(' ').unwrap_or(0);
                let prefix = &current_input[..=last_space_idx];
                let target = &current_input[last_space_idx + 1..];
                (prefix, target)
            }
        };

        // Perform completion
        let completion_result = AUTOCOMPLETE.with(|autocomplete| {
            autocomplete
                .borrow_mut()
                .complete(current_input, &current_path)
        });

        match completion_result {
            CompletionResult::None => {
                // No completions available - do nothing
            }
            CompletionResult::Single(completion) => {
                // Single completion - construct the full command
                let full_completion = if command_prefix.is_empty() {
                    // Completing a command
                    completion
                } else {
                    // Completing a path/argument - combine command prefix with completion
                    format!("{}{}", command_prefix, completion)
                };

                hidden_input.set_value(&full_completion);
                CURRENT_INPUT.with(|input| {
                    *input.borrow_mut() = full_completion.clone();
                });

                // Set cursor position to the end of the completed text
                let cursor_pos = full_completion.len();
                let _ = hidden_input.set_selection_range(cursor_pos as u32, cursor_pos as u32);

                line_buffer::update_input_state(full_completion, cursor_pos);
                terminal.render();
            }
            CompletionResult::Multiple(completions) => {
                // Multiple completions - find common prefix and show options
                if let Some(common_prefix) = find_common_prefix(&completions) {
                    // If there's a common prefix longer than current completion target, use it
                    if common_prefix.len() > completion_target.len() {
                        let full_completion = if command_prefix.is_empty() {
                            common_prefix
                        } else {
                            format!("{}{}", command_prefix, common_prefix)
                        };

                        hidden_input.set_value(&full_completion);
                        CURRENT_INPUT.with(|input| {
                            *input.borrow_mut() = full_completion.clone();
                        });

                        // Set cursor position to the end of the common prefix
                        let cursor_pos = full_completion.len();
                        let _ =
                            hidden_input.set_selection_range(cursor_pos as u32, cursor_pos as u32);

                        line_buffer::update_input_state(full_completion, cursor_pos);
                        terminal.render();
                        return;
                    }
                }

                // Show all available completions
                let prompt = terminal.get_current_prompt();
                line_buffer::add_command_line(&prompt, current_input);

                // Format completions nicely
                let completions_text = if completions.len() <= 10 {
                    // Show all completions on one line if there are few
                    completions.join("  ")
                } else {
                    // Show completions in columns if there are many
                    let mut output = String::new();
                    for (i, completion) in completions.iter().enumerate() {
                        if i > 0 && i % 4 == 0 {
                            output.push('\n');
                        } else if i > 0 {
                            output.push_str("  ");
                        }
                        output.push_str(completion);
                    }
                    output
                };

                line_buffer::add_output_lines(&completions_text, None);

                // Restore the input prompt
                Self::prepare_for_next_input(terminal, hidden_input);
                hidden_input.set_value(current_input);
                CURRENT_INPUT.with(|input| {
                    *input.borrow_mut() = current_input.to_string();
                });

                // Set cursor position to the end of the current input
                let cursor_pos = current_input.len();
                let _ = hidden_input.set_selection_range(cursor_pos as u32, cursor_pos as u32);

                line_buffer::update_input_state(current_input.to_string(), cursor_pos);
                terminal.render();
            }
        }
    }

    async fn handle_system_panic(terminal: &Terminal) {
        // Clear screen and set disabled mode
        line_buffer::clear_buffer();
        line_buffer::set_input_mode(InputMode::Disabled);

        let panic_lines = vec![
            ("⚠️  CRITICAL SYSTEM ERROR ⚠️", Some("error")),
            ("", None),
            ("Deleting root filesystem...", Some("warning")),
            ("rm: removing /usr... ████████████░░░░ 75%", Some("warning")),
            ("rm: removing /var... ██████████████░░ 87%", Some("warning")),
            (
                "rm: removing /etc... ████████████████ 100%",
                Some("warning"),
            ),
            ("", None),
            ("SYSTEM DESTROYED ☠️", Some("error")),
            ("", None),
            (
                "Just kidding! This is a just website, not your actual system.",
                Some("success"),
            ),
            ("Nice try though! 😉", Some("success")),
            ("", None),
            (
                "(Don't actually run 'sudo rm -rf /' on real systems!)",
                Some("warning"),
            ),
        ];

        for (line, color) in panic_lines {
            line_buffer::add_line(
                line.to_string(),
                LineType::System,
                color.map(|s| s.to_string()),
            );
            terminal.render();
            terminal.sleep(500).await;
        }

        terminal.sleep(2000).await;

        // Clear and show recovery message
        line_buffer::clear_buffer();
        line_buffer::add_line(
            "System restored! Terminal is back online.".to_string(),
            LineType::System,
            Some("success".to_string()),
        );
        line_buffer::add_line("".to_string(), LineType::Normal, None);
        terminal.render();
    }

    fn setup_cursor_blink(terminal: &Terminal) {
        let terminal_clone = terminal.clone();

        let blink_callback = Closure::wrap(Box::new(move || {
            // Only blink cursor if terminal is focused
            let is_focused = IS_FOCUSED.with(|focused| *focused.borrow());
            let state = line_buffer::get_terminal_state();

            if is_focused && state.input_mode == InputMode::Normal {
                terminal_clone.renderer.toggle_cursor();
                terminal_clone.render();
            }
        }) as Box<dyn FnMut()>);

        window()
            .unwrap()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                blink_callback.as_ref().unchecked_ref(),
                500, // Blink every 500ms
            )
            .unwrap();
        blink_callback.forget();
    }
}
