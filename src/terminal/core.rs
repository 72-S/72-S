use super::line_buffer;
use super::renderer::{LineOptions, TerminalRenderer};
use crate::commands::CommandHandler;
use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, CanvasRenderingContext2d, Document, HtmlCanvasElement};

#[derive(Clone)]
pub struct Terminal {
    pub renderer: TerminalRenderer,
    pub command_handler: CommandHandler,
    pub base_prompt: String,
}

impl Terminal {
    pub fn new(document: &Document) -> Self {
        let canvas = document
            .get_element_by_id("terminal")
            .expect("canvas not found")
            .dyn_into::<HtmlCanvasElement>()
            .expect("element is not a canvas");

        // Set fixed canvas dimensions to match CSS
        let canvas_width = 800;
        let canvas_height = 600;

        canvas.set_width(canvas_width);
        canvas.set_height(canvas_height);

        let context = canvas
            .get_context("2d")
            .expect("failed to get 2d context")
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .expect("failed to cast to CanvasRenderingContext2d");

        let renderer = TerminalRenderer::new(canvas, context);
        let command_handler = CommandHandler::new();
        let base_prompt = "objz@objz".to_string();

        // Initialize line buffer dimensions
        line_buffer::set_terminal_dimensions(
            renderer.max_chars_per_line(),
            renderer.max_visible_lines(),
        );

        Self {
            renderer,
            command_handler,
            base_prompt,
        }
    }

    pub fn get_current_prompt(&self) -> String {
        let cwd = self.command_handler.get_current_directory();
        let display_path = if cwd == "/home/objz" {
            "~".to_string()
        } else if cwd.starts_with("/home/objz/") {
            format!("~{}", &cwd["/home/objz".len()..])
        } else {
            cwd
        };

        format!("{}:{}$ ", self.base_prompt, display_path)
    }

    pub async fn sleep(&self, ms: i32) {
        let promise = Promise::new(&mut |resolve, _reject| {
            let window = window().unwrap();
            let closure = Closure::once_into_js(move || {
                resolve.call0(&wasm_bindgen::JsValue::UNDEFINED).unwrap();
            });
            window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    ms,
                )
                .unwrap();
        });

        let _ = JsFuture::from(promise).await;
    }

    // Legacy compatibility methods
    pub async fn add_line(&self, text: &str, options: Option<LineOptions>) {
        self.renderer.add_line(text, options).await;
    }

    pub fn clear_output(&self) {
        self.renderer.clear_output();
    }

    pub fn prepare_for_input(&self) {
        // Update prompt with current directory
        let prompt = self.get_current_prompt();
        line_buffer::set_current_prompt(prompt);
        self.renderer.prepare_for_input();
    }

    pub fn finalize_input(&self, input: &str) {
        self.renderer.finalize_input(input);
    }

    pub fn update_input_display(&self, input: &str) {
        self.renderer.update_input_display(input);
    }

    pub fn draw_current_input_line(&self, input: &str) {
        self.renderer.draw_current_input_line(input);
    }

    pub fn draw_cursor(&self, input: &str) {
        self.renderer.draw_cursor_legacy(input);
    }

    // New methods for the enhanced line buffer system
    pub fn render(&self) {
        self.renderer.render();
    }

    pub fn add_output(&self, text: &str, color: Option<String>) {
        line_buffer::add_output_lines(text, color);
        self.renderer.render();
    }

    pub fn add_command_output(&self, command: &str, output: &str) {
        let prompt = self.get_current_prompt();
        line_buffer::add_command_line(&prompt, command);
        if !output.is_empty() {
            line_buffer::add_output_lines(output, None);
        }
        self.renderer.render();
    }

    pub fn scroll_up(&self, lines: usize) {
        line_buffer::scroll_up(lines);
        self.renderer.render();
    }

    pub fn scroll_down(&self, lines: usize) {
        line_buffer::scroll_down(lines);
        self.renderer.render();
    }

    pub fn reset_scroll(&self) {
        line_buffer::reset_scroll();
        self.renderer.render();
    }
}
