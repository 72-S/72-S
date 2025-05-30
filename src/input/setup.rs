use crate::input::history::CommandHistory;
use crate::terminal::Terminal;
use crate::utils::panic;
use crate::{input::autoscroll::ensure_autoscroll, terminal::renderer::LineOptions};
use js_sys::Promise;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{window, Event, HtmlTextAreaElement, KeyboardEvent};

impl Terminal {
    pub fn init_shell(&self) {
        self.create_prompt_input();
        self.show_prompt();
        self.setup_input_handlers();
    }

    fn create_prompt_input(&self) {
        let doc = window().unwrap().document().unwrap();

        let prompt_div = doc.create_element("div").unwrap();
        prompt_div.set_class_name("prompt-line");

        let label = doc.create_element("span").unwrap();
        label.set_class_name("prompt");
        label.set_text_content(Some(&self.get_current_prompt()));

        let textarea = doc
            .create_element("textarea")
            .unwrap()
            .dyn_into::<HtmlTextAreaElement>()
            .unwrap();
        textarea.set_id("terminal-input");
        textarea.set_class_name("terminal-input");
        textarea.set_attribute("autocomplete", "off").unwrap();
        textarea.set_attribute("spellcheck", "false").unwrap();
        textarea.set_attribute("rows", "1").unwrap();
        textarea.set_attribute("wrap", "soft").unwrap();

        prompt_div.append_child(&label).unwrap();
        prompt_div.append_child(&textarea).unwrap();
        self.canvas.append_child(&prompt_div).unwrap();

        ensure_autoscroll();
    }

    fn setup_input_handlers(&self) {
        self.setup_auto_resize();
        self.setup_typing_animation();
        self.setup_key_handler();
    }

    fn setup_auto_resize(&self) {
        let doc = window().unwrap().document().unwrap();
        let input = doc.get_element_by_id("terminal-input").unwrap();
        let clone = input.clone();

        let on_input = Closure::wrap(Box::new(move |_e: Event| {
            if let Some(tx) = clone.dyn_ref::<HtmlTextAreaElement>() {
                tx.style().set_property("height", "auto").unwrap();
                let scroll_h = tx.scroll_height();

                let line_h = 22;
                let max_lines = 10;

                let min_h = line_h;
                let max_h = line_h * max_lines;
                let h = scroll_h.max(min_h).min(max_h);

                tx.style()
                    .set_property("height", &format!("{}px", h))
                    .unwrap();

                let rows = (h / line_h).max(1);
                tx.set_attribute("rows", &rows.to_string()).unwrap();
            }
        }) as Box<dyn FnMut(_)>);

        input
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .unwrap();
        on_input.forget();
    }

    fn setup_typing_animation(&self) {
        let doc = window().unwrap().document().unwrap();
        let input = doc.get_element_by_id("terminal-input").unwrap();
        let clone = input.clone();

        let on_input = Closure::wrap(Box::new(move |_e: Event| {
            if let Some(tx) = clone.dyn_ref::<HtmlTextAreaElement>() {
                tx.set_class_name("terminal-input typing");
                let tx_clone = tx.clone();

                let rm_typing = Closure::wrap(Box::new(move || {
                    tx_clone.set_class_name("terminal-input");
                }) as Box<dyn FnMut()>);

                window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        rm_typing.as_ref().unchecked_ref(),
                        150,
                    )
                    .unwrap();
                rm_typing.forget();
            }
        }) as Box<dyn FnMut(_)>);

        input
            .add_event_listener_with_callback("input", on_input.as_ref().unchecked_ref())
            .unwrap();
        on_input.forget();
    }

    fn setup_key_handler(&self) {
        let doc = window().unwrap().document().unwrap();
        let input = doc
            .get_element_by_id("terminal-input")
            .unwrap()
            .dyn_into::<HtmlTextAreaElement>()
            .unwrap();

        let mut history = {
            let mut h = CommandHistory::new(50);
            for cmd in &["help", "clear", "ls"] {
                h.add(cmd.to_string());
            }
            h
        };

        let mut processor = self.command_handler.clone();
        let base_prompt = self.base_prompt.clone();
        let clone_in = input.clone();
        let term_clone = self.clone();

        let autocomplete = Rc::new(RefCell::new(
            crate::terminal::autocomplete::AutoComplete::new(),
        ));

        let handler = Closure::wrap(Box::new(move |ev: KeyboardEvent| match ev.key().as_str() {
            "Tab" => {
                ev.prevent_default();
                Self::handle_tab_completion(
                    &clone_in,
                    &autocomplete,
                    &processor,
                    &base_prompt,
                    &term_clone,
                );
            }

            "Enter" if !ev.shift_key() => {
                ev.prevent_default();
                Self::handle_enter(
                    &clone_in,
                    &mut history,
                    &mut processor,
                    &base_prompt,
                    &term_clone,
                );
            }

            "ArrowUp" => {
                ev.prevent_default();
                Self::handle_history_up(&clone_in, &mut history);
            }

            "ArrowDown" => {
                ev.prevent_default();
                Self::handle_history_down(&clone_in, &mut history);
            }

            _ => {}
        }) as Box<dyn FnMut(_)>);

        input
            .add_event_listener_with_callback("keydown", handler.as_ref().unchecked_ref())
            .unwrap();
        handler.forget();
    }

    fn handle_tab_completion(
        input: &HtmlTextAreaElement,
        autocomplete: &Rc<RefCell<crate::terminal::autocomplete::AutoComplete>>,
        processor: &crate::commands::CommandHandler,
        base_prompt: &str,
        terminal: &Terminal,
    ) {
        let current_input = input.value();
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
                if parts.len() <= 1 {
                    input.set_value(&format!("{} ", completion));
                } else {
                    let mut new_parts = parts[..parts.len() - 1].to_vec();
                    new_parts.push(&completion);
                    input.set_value(&format!("{} ", new_parts.join(" ")));
                }
                let text_length = input.value().len() as u32;
                let _ = input.set_selection_range(text_length, text_length);
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
                    if parts.len() <= 1 && common.len() > current_input.trim().len() {
                        input.set_value(&common);
                    } else if parts.len() > 1 {
                        let prefix = parts[..parts.len() - 1].join(" ");
                        input.set_value(&format!("{} {}", prefix, common));
                    }
                }
                ensure_autoscroll();
            }

            crate::terminal::autocomplete::CompletionResult::None => {}
        }
    }

    fn handle_enter(
        input: &HtmlTextAreaElement,
        history: &mut CommandHistory,
        processor: &mut crate::commands::CommandHandler,
        base_prompt: &str,
        terminal: &Terminal,
    ) {
        let val = input.value();
        if !val.trim().is_empty() {
            history.add(val.clone());
        }

        input.set_value("");
        input.style().set_property("height", "auto").unwrap();
        input.set_attribute("rows", "1").unwrap();

        let current_prompt = Self::build_prompt(processor, base_prompt);
        let line = format!("{}{}", current_prompt, val);

        let term_clone_for_command = terminal.clone();
        spawn_local(async move {
            term_clone_for_command
                .add_line(&line, Some(LineOptions::new().with_color("command")))
                .await;
        });
        ensure_autoscroll();

        let (result, directory_changed) = processor.handle(&val);

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

        if directory_changed {
            Self::update_prompt_display(processor, base_prompt);
        }
    }

    fn handle_history_up(input: &HtmlTextAreaElement, history: &mut CommandHistory) {
        if let Some(cmd) = history.previous() {
            input.set_value(&cmd);
            input.focus().unwrap();
            let len = cmd.len() as u32;
            let _ = input.set_selection_range(len, len);

            let input_clone = input.clone();
            let timeout = Closure::wrap(Box::new(move || {
                let _ = input_clone.set_selection_range(len, len);
            }) as Box<dyn FnMut()>);

            window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    timeout.as_ref().unchecked_ref(),
                    0,
                )
                .unwrap();
            timeout.forget();
        }
    }

    fn handle_history_down(input: &HtmlTextAreaElement, history: &mut CommandHistory) {
        if let Some(cmd) = history.next() {
            input.set_value(&cmd);
        } else {
            input.set_value("");
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
        });
    }

    fn update_prompt_display(processor: &crate::commands::CommandHandler, base_prompt: &str) {
        let doc = window().unwrap().document().unwrap();
        if let Some(prompt_el) = doc.query_selector(".prompt-line .prompt").unwrap() {
            let new_prompt = Self::build_prompt(processor, base_prompt);
            prompt_el.set_text_content(Some(&new_prompt));
        }
    }

    fn show_prompt(&self) {
        let doc = window().unwrap().document().unwrap();
        let input = doc
            .get_element_by_id("terminal-input")
            .unwrap()
            .dyn_into::<HtmlTextAreaElement>()
            .unwrap();
        input.focus().unwrap();
        ensure_autoscroll();
    }
}
