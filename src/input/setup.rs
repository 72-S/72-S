use crate::input::history::CommandHistory;
use crate::terminal::renderer::LineOptions;
use crate::terminal::Terminal;
use crate::utils::panic;
use js_sys::Promise;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{window, Event, HtmlTextAreaElement, KeyboardEvent};

// Add thread-local storage for current input
thread_local! {
    static CURRENT_INPUT: RefCell<String> = RefCell::new(String::new());
}

impl Terminal {
    pub fn init_shell(&self) {
        // Remove the DOM-based input creation
        // self.create_prompt_input();

        // Initialize canvas-based prompt
        self.prepare_for_input();

        self.setup_canvas_input_handlers();
    }

    fn setup_canvas_input_handlers(&self) {
        let mut history = {
            let mut h = CommandHistory::new(50);
            for cmd in &["help", "clear", "ls"] {
                h.add(cmd.to_string());
            }
            h
        };

        let mut processor = self.command_handler.clone();
        let base_prompt = self.base_prompt.clone();
        let term_clone = self.clone();

        let autocomplete = Rc::new(RefCell::new(
            crate::terminal::autocomplete::AutoComplete::new(),
        ));

        // Set up keydown event listener on the canvas
        let canvas_clone = self.canvas.clone();
        let handler = Closure::wrap(Box::new(move |ev: KeyboardEvent| {
            // Make sure canvas is focused
            let _ = canvas_clone.focus();

            match ev.key().as_str() {
                "Tab" => {
                    ev.prevent_default();
                    CURRENT_INPUT.with(|input| {
                        let current_input = input.borrow().clone();
                        Self::handle_canvas_tab_completion(
                            &current_input,
                            &autocomplete,
                            &processor,
                            &base_prompt,
                            &term_clone,
                        );
                    });
                }

                "Enter" => {
                    ev.prevent_default();
                    CURRENT_INPUT.with(|input| {
                        let current_input = input.borrow().clone();
                        Self::handle_canvas_enter(
                            &current_input,
                            &mut history,
                            &mut processor,
                            &base_prompt,
                            &term_clone,
                        );
                        input.borrow_mut().clear();
                        term_clone.prepare_for_input();
                    });
                }

                "Backspace" => {
                    ev.prevent_default();
                    CURRENT_INPUT.with(|input| {
                        let mut current_input = input.borrow_mut();
                        if !current_input.is_empty() {
                            current_input.pop();
                            term_clone.update_input_display(&current_input);
                        }
                    });
                }

                "ArrowUp" => {
                    ev.prevent_default();
                    if let Some(cmd) = history.previous() {
                        CURRENT_INPUT.with(|input| {
                            *input.borrow_mut() = cmd;
                            term_clone.update_input_display(&input.borrow());
                        });
                    }
                }

                "ArrowDown" => {
                    ev.prevent_default();
                    let cmd = history.next().unwrap_or_default();
                    CURRENT_INPUT.with(|input| {
                        *input.borrow_mut() = cmd;
                        term_clone.update_input_display(&input.borrow());
                    });
                }

                _ => {
                    // Handle regular character input
                    if ev.key().len() == 1 && !ev.ctrl_key() && !ev.alt_key() && !ev.meta_key() {
                        CURRENT_INPUT.with(|input| {
                            input.borrow_mut().push_str(&ev.key());
                            term_clone.update_input_display(&input.borrow());
                        });
                    }
                }
            }
        }) as Box<dyn FnMut(_)>);

        self.canvas
            .add_event_listener_with_callback("keydown", handler.as_ref().unchecked_ref())
            .unwrap();
        handler.forget();

        // Make sure canvas can receive focus
        self.canvas.set_attribute("tabindex", "0").unwrap();
        let _ = self.canvas.focus();
    }

    fn handle_canvas_tab_completion(
        current_input: &str,
        autocomplete: &Rc<RefCell<crate::terminal::autocomplete::AutoComplete>>,
        processor: &crate::commands::CommandHandler,
        base_prompt: &str,
        terminal: &Terminal,
    ) {
        let current_path = {
            use crate::commands::filesystem::CURRENT_PATH;
            CURRENT_PATH.lock().unwrap().clone()
        };

        let completion_result = autocomplete
            .borrow_mut()
            .complete(&current_input, &current_path);

        match completion_result {
            crate::terminal::autocomplete::CompletionResult::Single(completion) => {
                let parts: Vec<&str> = current_input.trim().split_whitespace().collect();
                let new_input = if parts.len() <= 1 {
                    format!("{} ", completion)
                } else {
                    let mut new_parts = parts[..parts.len() - 1].to_vec();
                    new_parts.push(&completion);
                    format!("{} ", new_parts.join(" "))
                };

                CURRENT_INPUT.with(|input| {
                    *input.borrow_mut() = new_input.clone();
                    terminal.update_input_display(&new_input);
                });
            }

            crate::terminal::autocomplete::CompletionResult::Multiple(matches) => {
                let current_prompt = Self::build_prompt(processor, base_prompt);
                let line = format!("{}{}", current_prompt, current_input);

                let term_clone_for_line = terminal.clone();
                spawn_local(async move {
                    term_clone_for_line
                        .add_line(&line, Some(LineOptions::new().with_color("command")))
                        .await;
                });

                Self::display_completions(&matches, terminal);

                if let Some(common) = crate::terminal::autocomplete::find_common_prefix(&matches) {
                    let parts: Vec<&str> = current_input.trim().split_whitespace().collect();
                    let new_input = if parts.len() <= 1 && common.len() > current_input.trim().len()
                    {
                        common
                    } else if parts.len() > 1 {
                        let prefix = parts[..parts.len() - 1].join(" ");
                        format!("{} {}", prefix, common)
                    } else {
                        current_input.to_string()
                    };

                    CURRENT_INPUT.with(|input| {
                        *input.borrow_mut() = new_input.clone();
                        terminal.prepare_for_input();
                        terminal.update_input_display(&new_input);
                    });
                }
            }

            crate::terminal::autocomplete::CompletionResult::None => {}
        }
    }

    fn handle_canvas_enter(
        current_input: &str,
        history: &mut CommandHistory,
        processor: &mut crate::commands::CommandHandler,
        base_prompt: &str,
        terminal: &Terminal,
    ) {
        if !current_input.trim().is_empty() {
            history.add(current_input.to_string());
        }

        let current_prompt = Self::build_prompt(processor, base_prompt);
        let line = format!("{}{}", current_prompt, current_input);

        let term_clone_for_command = terminal.clone();
        spawn_local(async move {
            term_clone_for_command
                .add_line(&line, Some(LineOptions::new().with_color("command")))
                .await;
        });

        let (result, directory_changed) = processor.handle(&current_input);

        match result.as_str() {
            "CLEAR_SCREEN" => terminal.clear_output(),
            "SYSTEM_PANIC" => {
                let panic_clone = terminal.clone();
                spawn_local(async move {
                    panic::system_panic(&panic_clone).await;
                });
            }
            _ => {
                Self::display_command_output(&result, terminal);
            }
        }

        // Note: Directory change handling for DOM-based prompt is removed
        // since we're using canvas-based input now
    }

    fn build_prompt(processor: &crate::commands::CommandHandler, base_prompt: &str) -> String {
        let cwd = processor.get_current_directory();
        let display_path = if cwd == "/home/objz" {
            "~".to_string()
        } else if cwd.starts_with("/home/objz/") {
            format!("~{}", &cwd["/home/objz".len()..])
        } else {
            cwd
        };
        format!("{}:{}$ ", base_prompt, display_path)
    }

    fn display_completions(matches: &[String], terminal: &Terminal) {
        let matches_per_line = 4;
        let mut output = String::new();

        for chunk in matches.chunks(matches_per_line) {
            let row = chunk
                .iter()
                .map(|s| format!("{:<20}", s))
                .collect::<Vec<_>>()
                .join("");
            output.push_str(&row);
            output.push('\n');
        }

        let term_clone_for_output = terminal.clone();
        spawn_local(async move {
            for line in output.lines() {
                if !line.trim().is_empty() {
                    term_clone_for_output
                        .add_line(&line, Some(LineOptions::new().with_color("completion")))
                        .await;
                }
            }
            // After showing completions, prepare for new input
            term_clone_for_output.prepare_for_input();
        });
    }

    fn display_command_output(result: &str, terminal: &Terminal) {
        let term_clone_for_output = terminal.clone();
        let lines: Vec<String> = result.lines().map(str::to_owned).collect();

        spawn_local(async move {
            for line in lines {
                term_clone_for_output.add_line(&line, None).await;

                let promise = Promise::new(&mut |resolve, _| {
                    window()
                        .unwrap()
                        .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 0)
                        .unwrap();
                });
                let _ = JsFuture::from(promise).await;
            }
            // After command output, prepare for new input
            term_clone_for_output.prepare_for_input();
        });
    }

    // Remove all the DOM-based methods since we're using canvas now
    // create_prompt_input, setup_input_handlers, setup_auto_resize,
    // setup_typing_animation, setup_key_handler, handle_history_up,
    // handle_history_down, update_prompt_display, show_prompt
}
