use crate::input::autoscroll::ensure_autoscroll;
use crate::input::history::CommandHistory;
use crate::terminal::Terminal;
use crate::utils::{
    dom::{append_line, clear_output},
    panic::system_panic,
};
use js_sys::Promise;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{window, Event, HtmlTextAreaElement, KeyboardEvent};

impl Terminal {
    pub fn setup_input_system(&self) {
        self.create_prompt_input_with_cursor();
        self.show_prompt();
        self.setup_auto_resize();
        self.setup_typing_animation();
        self.setup_key_handler();
    }

    fn create_prompt_input_with_cursor(&self) {
        let doc = window().unwrap().document().unwrap();
        let body = doc.get_element_by_id("terminal-body").unwrap();

        let prompt_div = doc.create_element("div").unwrap();
        prompt_div.set_class_name("prompt-line");

        let label = doc.create_element("span").unwrap();
        label.set_class_name("prompt");
        label.set_text_content(Some(&self.get_current_prompt()));

        let input_wrapper = doc.create_element("div").unwrap();
        input_wrapper.set_class_name("input-container");

        let textarea = doc
            .create_element("textarea")
            .unwrap()
            .dyn_into::<HtmlTextAreaElement>()
            .unwrap();
        textarea.set_id("terminal-input");
        textarea.set_class_name("terminal-input-field");
        textarea.set_attribute("autocomplete", "off").unwrap();
        textarea.set_attribute("spellcheck", "false").unwrap();
        textarea.set_attribute("rows", "1").unwrap();
        textarea.set_attribute("wrap", "soft").unwrap();

        input_wrapper.append_child(&textarea).unwrap();
        prompt_div.append_child(&label).unwrap();
        prompt_div.append_child(&input_wrapper).unwrap();
        body.append_child(&prompt_div).unwrap();

        ensure_autoscroll();
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
                let min_h = line_h;
                let max_h = line_h * 10;
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
                tx.set_class_name("terminal-input-field typing");
                let tx_clone = tx.clone();
                let rm_typing = Closure::wrap(Box::new(move || {
                    tx_clone.set_class_name("terminal-input-field");
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
        let out_el = self.output_element.clone();
        let mut history = {
            let mut h = CommandHistory::new(50);
            for cmd in &["help", "clear", "ls"] {
                h.add(cmd.to_string());
            }
            h
        };
        let mut processor = self.command_processor.clone();
        let base_prompt = self.base_prompt.clone();
        let clone_in = input.clone();

        use std::cell::RefCell;
        use std::rc::Rc;
        let autocomplete = Rc::new(RefCell::new(
            crate::terminal::autocomplete::AutoComplete::new(),
        ));

        let handler = Closure::wrap(Box::new(move |ev: KeyboardEvent| match ev.key().as_str() {
            "Tab" => {
                ev.prevent_default();
                let current_input = clone_in.value();
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
                            clone_in.set_value(&format!("{} ", completion));
                        } else {
                            let mut new_parts = parts[..parts.len() - 1].to_vec();
                            new_parts.push(&completion);
                            clone_in.set_value(&format!("{} ", new_parts.join(" ")));
                        }
                        let text_length = clone_in.value().len() as u32;
                        let _ = clone_in.set_selection_range(text_length, text_length);
                    }
                    crate::terminal::autocomplete::CompletionResult::Multiple(matches) => {
                        let current_prompt = {
                            let cwd = processor.get_current_directory();
                            let display_path = if cwd == "/home/objz" {
                                "~".to_string()
                            } else if cwd.starts_with("/home/objz/") {
                                format!("~{}", &cwd["/home/objz".len()..])
                            } else {
                                cwd
                            };
                            format!("{}:{}$ ", base_prompt, display_path)
                        };
                        let line = format!("{}{}", current_prompt, current_input);
                        append_line(&out_el, &line, Some("command"));

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
                        for line in output.lines() {
                            if !line.trim().is_empty() {
                                append_line(&out_el, line, Some("completion"));
                            }
                        }
                        if let Some(common) =
                            crate::terminal::autocomplete::find_common_prefix(&matches)
                        {
                            let parts: Vec<&str> =
                                current_input.trim().split_whitespace().collect();
                            if parts.len() <= 1 && common.len() > current_input.trim().len() {
                                clone_in.set_value(&common);
                            } else if parts.len() > 1 {
                                let prefix = parts[..parts.len() - 1].join(" ");
                                clone_in.set_value(&format!("{} {}", prefix, common));
                            }
                        }
                        ensure_autoscroll();
                    }
                    crate::terminal::autocomplete::CompletionResult::None => {}
                }
            }

            "Enter" if !ev.shift_key() => {
                ev.prevent_default();
                let val = clone_in.value();
                if !val.trim().is_empty() {
                    history.add(val.clone());
                }
                clone_in.set_value("");
                clone_in.style().set_property("height", "auto").unwrap();
                clone_in.set_attribute("rows", "1").unwrap();

                let current_prompt = {
                    let cwd = processor.get_current_directory();
                    let display_path = if cwd == "/home/objz" {
                        "~".to_string()
                    } else if cwd.starts_with("/home/objz/") {
                        format!("~{}", &cwd["/home/objz".len()..])
                    } else {
                        cwd
                    };
                    format!("{}:{}$ ", base_prompt, display_path)
                };

                let line = format!("{}{}", current_prompt, val);
                append_line(&out_el, &line, Some("command"));
                ensure_autoscroll();

                let (result, directory_changed) = processor.handle(&val);

                match result.as_str() {
                    "CLEAR_SCREEN" => clear_output(&out_el),
                    "SYSTEM_PANIC" => {
                        let out_clone = out_el.clone();
                        spawn_local(async move {
                            system_panic(&out_clone).await;
                        });
                    }
                    _ => {
                        let out_clone = out_el.clone();
                        let lines: Vec<String> = result.lines().map(str::to_owned).collect();
                        spawn_local(async move {
                            for line in lines {
                                append_line(&out_clone, &line, None);
                                // ensure_autoscroll();
                                let promise = Promise::new(&mut |resolve, _| {
                                    window()
                                        .unwrap()
                                        .set_timeout_with_callback_and_timeout_and_arguments_0(
                                            &resolve, 0,
                                        )
                                        .unwrap();
                                });
                                let _ = JsFuture::from(promise).await;
                            }
                        });
                    }
                }

                if directory_changed {
                    let doc = window().unwrap().document().unwrap();
                    if let Some(prompt_el) = doc.query_selector(".prompt-line .prompt").unwrap() {
                        let new_cwd = processor.get_current_directory();
                        let new_display = if new_cwd == "/home/objz" {
                            "~".to_string()
                        } else if new_cwd.starts_with("/home/objz/") {
                            format!("~{}", &new_cwd["/home/objz".len()..])
                        } else {
                            new_cwd
                        };
                        prompt_el
                            .set_text_content(Some(&format!("{}:{}$ ", base_prompt, new_display)));
                    }
                }
            }

            "ArrowUp" => {
                ev.prevent_default();
                if let Some(cmd) = history.previous() {
                    clone_in.set_value(&cmd);
                    clone_in.focus().unwrap();
                    let len = cmd.len() as u32;
                    let _ = clone_in.set_selection_range(len, len);
                    // clone for timeout instead of moving original
                    let input_clone = clone_in.clone();
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

            "ArrowDown" => {
                ev.prevent_default();
                if let Some(cmd) = history.next() {
                    clone_in.set_value(&cmd);
                } else {
                    clone_in.set_value("");
                }
            }
            _ => {}
        }) as Box<dyn FnMut(_)>);

        input
            .add_event_listener_with_callback("keydown", handler.as_ref().unchecked_ref())
            .unwrap();
        handler.forget();
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
