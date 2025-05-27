use crate::animator::Animator;
use crate::commands::CommandProcessor;
use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local; // <-- bring spawn_local into scope
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Document, Element, HtmlElement, HtmlInputElement, KeyboardEvent};

pub struct Terminal {
    output_element: Element,
    input_element: HtmlInputElement,
    command_processor: CommandProcessor,
    prompt: String,
    animator: Animator,
}

impl Terminal {
    pub fn new(document: &Document) -> Self {
        let output_element = document.get_element_by_id("terminal-output").unwrap();
        let input_element = document
            .get_element_by_id("terminal-input")
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();

        let command_processor = CommandProcessor::new();
        let prompt = "objz@portfolio:~$ ".to_string();
        let animator = Animator::new();

        Self {
            output_element,
            input_element,
            command_processor,
            prompt,
            animator,
        }
    }

    pub async fn start_hacking_intro(&self) {
        self.clear_output();

        // Boot sequence
        let boot_lines = vec![
            "Initializing objz.dev terminal...",
            "Loading system modules...",
            "Establishing secure connection...",
            "",
        ];

        for line in boot_lines {
            self.add_line_with_typing(line, 50).await;
            self.sleep(200).await;
        }

        // ASCII Logo
        let logo_lines = vec![
            "  ___  ___ ___  ____",
            " / _ \\| _ ) _ \\|_  /",
            "| (_) | _ \\   / / / ",
            " \\___/|___/_|_\\/___|",
            "",
            "PORTFOLIO v3.0.0 - ADVANCED TERMINAL",
            "",
        ];

        for line in logo_lines {
            if line.starts_with(" ")
                && (line.contains("/") || line.contains("\\") || line.contains("|"))
            {
                self.add_line_with_color_and_typing(line, "cyan", 30).await;
            } else if line.contains("PORTFOLIO") {
                self.add_line_with_color_and_typing(line, "green", 50).await;
            } else {
                self.add_line_with_typing(line, 40).await;
            }
            self.sleep(100).await;
        }

        // System checks
        let system_checks = vec![
            ("Scanning for vulnerabilities...", "[OK]", "green"),
            ("Loading project database...", "[OK]", "green"),
            ("Initializing command processor...", "[OK]", "green"),
            ("Setting up animations...", "[OK]", "green"),
            ("Establishing matrix connection...", "[CONNECTED]", "yellow"),
        ];

        for (task, status, color) in system_checks {
            self.add_line_with_typing(task, 30).await;
            self.sleep(500).await;

            // Animate loading bar
            for progress in (0..=100).step_by(25) {
                let bar = self.animator.get_loading_bar(progress);
                self.update_last_line(&format!("{} {}", task, bar)).await;
                self.sleep(50).await;
            }

            self.update_last_line(&format!("{} {}", task, status)).await;
            self.apply_color_to_last_line(color);
            self.sleep(200).await;
        }

        self.add_line("").await;
        self.add_line_with_color_and_typing("ACCESS GRANTED", "green", 100)
            .await;
        self.sleep(500).await;

        self.add_line("").await;
        self.add_line_with_typing("Welcome to objz's Interactive Portfolio Terminal", 40)
            .await;
        self.add_line_with_typing("Type 'help' to see available commands", 40)
            .await;
        self.add_line("").await;
    }

    pub async fn start_interactive_shell(&self) {
        self.show_prompt();
        self.setup_input_handler();
    }

    fn setup_input_handler(&self) {
        let output_element = self.output_element.clone();
        let input_element = self.input_element.clone();
        let mut command_processor = self.command_processor.clone();
        let prompt = self.prompt.clone();

        let closure = Closure::wrap(Box::new(move |event: KeyboardEvent| {
            if event.key() == "Enter" {
                let input_value = input_element.value();
                input_element.set_value("");

                // Add command to output
                let command_line = format!("{}{}", prompt, input_value);
                append_line(&output_element, &command_line, Some("command"));

                // Process command
                let output = command_processor.process_command(&input_value);

                // Handle special commands
                if output == "CLEAR_SCREEN" {
                    clear_output(&output_element);
                } else if output == "SYSTEM_PANIC" {
                    spawn_local({
                        let output_element = output_element.clone();
                        let prompt = prompt.clone();
                        async move {
                            show_system_panic(&output_element).await;
                            append_prompt(&output_element, &prompt);
                        }
                    });
                } else {
                    // Regular output
                    for line in output.lines() {
                        if line.starts_with("ERROR") || line.contains("not found") {
                            append_line(&output_element, line, Some("error"));
                        } else if line.contains("[OK]") || line.contains("SUCCESS") {
                            append_line(&output_element, line, Some("success"));
                        } else if line.contains("GitHub:") || line.contains("http") {
                            append_line(&output_element, line, Some("info"));
                        } else if line.contains("█") || line.contains("╔") || line.contains("┌")
                        {
                            append_line(&output_element, line, Some("ascii"));
                        } else {
                            append_line(&output_element, line, None);
                        }
                    }
                }

                // Show new prompt
                append_prompt(&output_element, &prompt);
                scroll_to_bottom(&output_element);
            }
        }) as Box<dyn FnMut(_)>);

        self.input_element
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .unwrap();

        closure.forget();
    }

    async fn add_line(&self, text: &str) {
        append_line(&self.output_element, text, None);
        scroll_to_bottom(&self.output_element);
    }

    async fn add_line_with_typing(&self, text: &str, delay_ms: i32) {
        let div = create_div(text, None);
        self.output_element.append_child(&div).unwrap();

        // Animate typing effect
        div.set_text_content(Some(""));
        for (i, _) in text.chars().enumerate() {
            let partial = &text[..=text.char_indices().nth(i).unwrap().0];
            div.set_text_content(Some(partial));
            scroll_to_bottom(&self.output_element);
            self.sleep(delay_ms).await;
        }
    }

    async fn add_line_with_color_and_typing(&self, text: &str, color: &str, delay_ms: i32) {
        let div = create_div(text, Some(color));
        self.output_element.append_child(&div).unwrap();

        // Animate typing effect
        div.set_text_content(Some(""));
        for (i, _) in text.chars().enumerate() {
            let partial = &text[..=text.char_indices().nth(i).unwrap().0];
            div.set_text_content(Some(partial));
            scroll_to_bottom(&self.output_element);
            self.sleep(delay_ms).await;
        }
    }

    async fn update_last_line(&self, text: &str) {
        if let Some(last_child) = self.output_element.last_element_child() {
            last_child.set_text_content(Some(text));
            scroll_to_bottom(&self.output_element);
        }
    }

    fn apply_color_to_last_line(&self, color: &str) {
        if let Some(last_child) = self.output_element.last_element_child() {
            last_child.set_class_name(color);
        }
    }

    fn show_prompt(&self) {
        append_prompt(&self.output_element, &self.prompt);
        self.input_element.focus().unwrap();
    }

    fn clear_output(&self) {
        self.output_element.set_inner_html("");
    }

    async fn sleep(&self, ms: i32) {
        let promise = Promise::new(&mut |resolve, _| {
            let window = window().unwrap();
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
        });

        let _ = JsFuture::from(promise).await;
    }
}

// Helper functions
fn create_div(text: &str, class: Option<&str>) -> Element {
    let div = window()
        .unwrap()
        .document()
        .unwrap()
        .create_element("div")
        .unwrap();
    div.set_text_content(Some(text));

    if let Some(class) = class {
        div.set_class_name(class);
    }

    div
}

fn append_line(element: &Element, text: &str, class: Option<&str>) {
    let div = create_div(text, class);
    element.append_child(&div).unwrap();
}

fn append_prompt(element: &Element, prompt: &str) {
    let div = window()
        .unwrap()
        .document()
        .unwrap()
        .create_element("div")
        .unwrap();
    div.set_class_name("prompt-line");
    div.set_inner_html(&format!("<span class='prompt'>{}</span>", prompt));
    element.append_child(&div).unwrap();
}

fn scroll_to_bottom(element: &Element) {
    if let Ok(html_element) = element.clone().dyn_into::<HtmlElement>() {
        html_element.set_scroll_top(html_element.scroll_height());
    }
}

fn clear_output(element: &Element) {
    element.set_inner_html("");
}

async fn show_system_panic(element: &Element) {
    // Clear screen and show panic
    clear_output(element);

    let panic_lines = vec![
        "⚠️  CRITICAL SYSTEM ERROR ⚠️",
        "",
        "Deleting root filesystem...",
        "rm: removing /usr... ████████████░░░░ 75%",
        "rm: removing /var... ██████████████░░ 87%",
        "rm: removing /etc... ████████████████ 100%",
        "",
        "SYSTEM DESTROYED ☠️",
        "",
        "Just kidding! This is a portfolio website, not your actual system.",
        "Nice try though! 😉",
        "",
        "(Don't actually run 'sudo rm -rf /' on real systems!)",
        "",
    ];

    for line in panic_lines {
        if line.contains("⚠️") || line.contains("☠️") {
            append_line(element, line, Some("error"));
        } else if line.contains("████") {
            append_line(element, line, Some("warning"));
        } else {
            append_line(element, line, None);
        }

        // Sleep for dramatic effect
        let promise = Promise::new(&mut |resolve, _| {
            let window = window().unwrap();
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 300);
        });
        let _ = JsFuture::from(promise).await;
    }

    // Wait a bit more before restoring
    let promise = Promise::new(&mut |resolve, _| {
        let window = window().unwrap();
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 2000);
    });
    let _ = JsFuture::from(promise).await;

    // Restore terminal
    clear_output(element);
    append_line(
        element,
        "System restored! Terminal is back online.",
        Some("success"),
    );
    append_line(element, "", None);
}
