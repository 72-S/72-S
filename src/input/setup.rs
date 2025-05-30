use crate::commands::CommandHandler;
use crate::terminal::{line_buffer, InputMode, LineType, Terminal};
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, HtmlInputElement, KeyboardEvent};

thread_local! {
    static CURRENT_INPUT: RefCell<String> = RefCell::new(String::new());
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
                        Self::handle_tab_completion(&terminal);
                    }
                    _ => {}
                }
            }) as Box<dyn FnMut(_)>)
        };

        hidden_input
            .add_event_listener_with_callback("keydown", keydown_callback.as_ref().unchecked_ref())
            .unwrap();
        keydown_callback.forget();

        // Set up cursor blinking
        Self::setup_cursor_blink(&terminal_clone);

        // Initialize terminal
        terminal.prepare_for_input();
        let _ = hidden_input.focus();
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

    fn handle_tab_completion(terminal: &Terminal) {
        // For now, just render to maintain state
        // TODO: Implement proper tab completion
        terminal.render();
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
            terminal_clone.renderer.toggle_cursor();
            let state = line_buffer::get_terminal_state();
            if state.input_mode == InputMode::Normal {
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
