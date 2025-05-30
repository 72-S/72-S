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
use web_sys::{window, CustomEvent, Event, HtmlInputElement, KeyboardEvent};

// Add thread-local storage for current input
thread_local! {
    static CURRENT_INPUT: RefCell<String> = RefCell::new(String::new());
    static INPUT_HANDLER_ATTACHED: RefCell<bool> = RefCell::new(false);
}

impl Terminal {
    pub fn init_shell(&self) {
        // Check if handler is already attached to prevent duplicates
        INPUT_HANDLER_ATTACHED.with(|attached| {
            if *attached.borrow() {
                return; // Already attached, don't attach again
            }
            *attached.borrow_mut() = true;
        });

        // Create hidden input field for proper input handling
        self.create_hidden_input();

        // Initialize canvas-based prompt display
        self.prepare_for_input();

        self.setup_hybrid_input_handlers();
        self.setup_3d_integration();
    }

    fn create_hidden_input(&self) {
        let doc = window().unwrap().document().unwrap();

        // Create a hidden input element
        let input = doc
            .create_element("input")
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();

        input.set_id("terminal-hidden-input");
        input.set_type("text");
        input.set_attribute("autocomplete", "off").unwrap();
        input.set_attribute("spellcheck", "false").unwrap();

        // Style to make it invisible but still focusable
        input.style().set_property("position", "absolute").unwrap();
        input.style().set_property("left", "-9999px").unwrap();
        input.style().set_property("top", "-9999px").unwrap();
        input.style().set_property("width", "1px").unwrap();
        input.style().set_property("height", "1px").unwrap();
        input.style().set_property("opacity", "0").unwrap();
        input
            .style()
            .set_property("pointer-events", "none")
            .unwrap();

        // Add to document body
        doc.body().unwrap().append_child(&input).unwrap();
    }

    fn setup_3d_integration(&self) {
        let doc = window().unwrap().document().unwrap();
        let hidden_input = doc
            .get_element_by_id("terminal-hidden-input")
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();

        // Listen for terminalFocus events from your 3D model
        let hidden_input_clone = hidden_input.clone();
        let focus_handler = Closure::wrap(Box::new(move |_: CustomEvent| {
            web_sys::console::log_1(&"Terminal focus event received".into());
            let _ = hidden_input_clone.focus();
        }) as Box<dyn FnMut(_)>);

        window()
            .unwrap()
            .add_event_listener_with_callback(
                "terminalFocus",
                focus_handler.as_ref().unchecked_ref(),
            )
            .unwrap();
        focus_handler.forget();

        // Listen for terminalBlur events from your 3D model
        let hidden_input_clone2 = hidden_input.clone();
        let blur_handler = Closure::wrap(Box::new(move |_: CustomEvent| {
            web_sys::console::log_1(&"Terminal blur event received".into());
            hidden_input_clone2.blur().unwrap();
        }) as Box<dyn FnMut(_)>);

        window()
            .unwrap()
            .add_event_listener_with_callback("terminalBlur", blur_handler.as_ref().unchecked_ref())
            .unwrap();
        blur_handler.forget();

        // Also handle direct canvas clicks as fallback
        let canvas_clone = self.canvas.clone();
        let hidden_input_clone3 = hidden_input.clone();
        let canvas_click_handler = Closure::wrap(Box::new(move |_: Event| {
            web_sys::console::log_1(&"Canvas clicked directly".into());
            let _ = hidden_input_clone3.focus();
        }) as Box<dyn FnMut(_)>);

        canvas_clone
            .add_event_listener_with_callback(
                "click",
                canvas_click_handler.as_ref().unchecked_ref(),
            )
            .unwrap();
        canvas_click_handler.forget();
    }

    fn setup_hybrid_input_handlers(&self) {
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

        // Get the hidden input element
        let doc = window().unwrap().document().unwrap();
        let hidden_input = doc
            .get_element_by_id("terminal-hidden-input")
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();

        // Set up input event for real-time display updates
        let hidden_input_clone = hidden_input.clone();
        let term_clone_for_input = term_clone.clone();
        let input_handler = Closure::wrap(Box::new(move |_: Event| {
            let current_value = hidden_input_clone.value();
            CURRENT_INPUT.with(|input| {
                *input.borrow_mut() = current_value.clone();
            });
            term_clone_for_input.update_input_display(&current_value);
        }) as Box<dyn FnMut(_)>);

        hidden_input
            .add_event_listener_with_callback("input", input_handler.as_ref().unchecked_ref())
            .unwrap();
        input_handler.forget();

        // Set up keydown event for special keys
        let hidden_input_clone2 = hidden_input.clone();
        let key_handler = Closure::wrap(Box::new(move |ev: KeyboardEvent| {
            match ev.key().as_str() {
                "Tab" => {
                    ev.prevent_default();
                    let current_input = hidden_input_clone2.value();
                    Self::handle_hybrid_tab_completion(
                        &current_input,
                        &autocomplete,
                        &processor,
                        &base_prompt,
                        &term_clone,
                        &hidden_input_clone2,
                    );
                }

                "Enter" => {
                    ev.prevent_default();
                    let current_input = hidden_input_clone2.value();
                    Self::handle_hybrid_enter(
                        &current_input,
                        &mut history,
                        &mut processor,
                        &base_prompt,
                        &term_clone,
                        &hidden_input_clone2,
                    );
                }

                "ArrowUp" => {
                    ev.prevent_default();
                    if let Some(cmd) = history.previous() {
                        hidden_input_clone2.set_value(&cmd);
                        CURRENT_INPUT.with(|input| {
                            *input.borrow_mut() = cmd.clone();
                        });
                        term_clone.update_input_display(&cmd);
                    }
                }

                "ArrowDown" => {
                    ev.prevent_default();
                    let cmd = history.next().unwrap_or_default();
                    hidden_input_clone2.set_value(&cmd);
                    CURRENT_INPUT.with(|input| {
                        *input.borrow_mut() = cmd.clone();
                    });
                    term_clone.update_input_display(&cmd);
                }

                _ => {
                    // Let other keys be handled by the input event
                }
            }
        }) as Box<dyn FnMut(_)>);

        hidden_input
            .add_event_listener_with_callback("keydown", key_handler.as_ref().unchecked_ref())
            .unwrap();
        key_handler.forget();

        // Focus the hidden input initially
        let _ = hidden_input.focus();
    }

    fn handle_hybrid_tab_completion(
        current_input: &str,
        autocomplete: &Rc<RefCell<crate::terminal::autocomplete::AutoComplete>>,
        processor: &crate::commands::CommandHandler,
        base_prompt: &str,
        terminal: &Terminal,
        hidden_input: &HtmlInputElement,
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

                hidden_input.set_value(&new_input);
                CURRENT_INPUT.with(|input| {
                    *input.borrow_mut() = new_input.clone();
                });
                terminal.update_input_display(&new_input);
            }

            crate::terminal::autocomplete::CompletionResult::Multiple(matches) => {
                // FIXED: Properly finalize current input before showing completions
                terminal.finalize_input(&current_input);

                Self::display_completions(&matches, terminal, hidden_input);

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

                    hidden_input.set_value(&new_input);
                    CURRENT_INPUT.with(|input| {
                        *input.borrow_mut() = new_input.clone();
                    });
                    terminal.prepare_for_input();
                    terminal.update_input_display(&new_input);
                }
            }

            crate::terminal::autocomplete::CompletionResult::None => {}
        }
    }

    fn handle_hybrid_enter(
        current_input: &str,
        history: &mut CommandHistory,
        processor: &mut crate::commands::CommandHandler,
        base_prompt: &str,
        terminal: &Terminal,
        hidden_input: &HtmlInputElement,
    ) {
        // FIXED: Properly finalize the input first
        terminal.finalize_input(&current_input);

        if !current_input.trim().is_empty() {
            history.add(current_input.to_string());
        }

        // Clear the hidden input
        hidden_input.set_value("");
        CURRENT_INPUT.with(|input| {
            input.borrow_mut().clear();
        });

        let (result, _directory_changed) = processor.handle(&current_input);

        match result.as_str() {
            "CLEAR_SCREEN" => {
                // FIXED: Clear screen properly and prepare for input
                terminal.clear_output(); // This now calls prepare_for_input() internally
            }
            "SYSTEM_PANIC" => {
                let panic_clone = terminal.clone();
                spawn_local(async move {
                    panic::system_panic(&panic_clone).await;
                });
            }
            _ => {
                Self::display_command_output(&result, terminal, &hidden_input);
            }
        }
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

    fn display_completions(
        matches: &[String],
        terminal: &Terminal,
        hidden_input: &HtmlInputElement,
    ) {
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
        let hidden_input_clone = hidden_input.clone();
        spawn_local(async move {
            for line in output.lines() {
                if !line.trim().is_empty() {
                    term_clone_for_output
                        .add_line(&line, Some(LineOptions::new().with_color("completion")))
                        .await;
                }
            }
            // After showing completions, prepare for new input and refocus
            term_clone_for_output.prepare_for_input();
            let _ = hidden_input_clone.focus();
        });
    }

    fn display_command_output(result: &str, terminal: &Terminal, hidden_input: &HtmlInputElement) {
        let term_clone_for_output = terminal.clone();
        let hidden_input_clone = hidden_input.clone();
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
            // After command output, prepare for new input and refocus
            term_clone_for_output.prepare_for_input();
            let _ = hidden_input_clone.focus();
        });
    }
}
