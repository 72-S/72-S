use super::line_buffer::{self, BufferLine, InputMode, LineType, TerminalState};
use js_sys::Promise;
use std::cell::Cell;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};

#[derive(Default)]
pub struct LineOptions {
    pub color: Option<String>,
    pub boot_animation: bool,
    pub typing_speed: Option<i32>,
}

impl LineOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_color(mut self, color: &str) -> Self {
        self.color = Some(color.to_string());
        self
    }

    pub fn with_boot_animation(mut self) -> Self {
        self.boot_animation = true;
        self
    }

    pub fn with_typing(mut self, speed: i32) -> Self {
        self.typing_speed = Some(speed);
        self
    }
}

pub struct TerminalRenderer {
    pub canvas: HtmlCanvasElement,
    pub context: CanvasRenderingContext2d,
    pub y: Cell<f64>,
    pub width: i32,
    pub height: i32,
    pub line_height: f64,
    pub char_width: f64,
    pub font_size: i32,
    pub cursor_blink_state: Cell<bool>,
}

impl TerminalRenderer {
    pub fn new(canvas: HtmlCanvasElement, context: CanvasRenderingContext2d) -> Self {
        let width = canvas.width() as i32;
        let height = canvas.height() as i32;
        let font_size = 14;
        let line_height = font_size as f64 + 6.0; // Add some line spacing
        let char_width = font_size as f64 * 0.6; // Monospace character width approximation

        // Configure context
        context.set_font(&format!("{}px 'Courier New', monospace", font_size));
        context.set_text_baseline("top");
        context.set_image_smoothing_enabled(false);

        Self {
            canvas,
            context,
            y: Cell::new(20.0),
            width,
            height,
            line_height,
            char_width,
            font_size,
            cursor_blink_state: Cell::new(true),
        }
    }

    /// Legacy add_line method for compatibility
    pub async fn add_line(&self, text: &str, options: Option<LineOptions>) {
        let opts = options.unwrap_or_default();

        if opts.boot_animation {
            self.boot(text, &opts).await;
        } else if let Some(speed) = opts.typing_speed {
            self.typing(text, speed, &opts).await;
        } else {
            self.simple(text, &opts).await;
        }
    }

    async fn boot(&self, task: &str, opts: &LineOptions) {
        let current_y = self.y.get();

        let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        for &spin_char in &spinner[0..4] {
            let text = format!("{} {}", task, spin_char);
            self.clear_line_at_y(current_y);
            self.draw_text(&text, 10.0, current_y, opts.color.as_deref());
            self.sleep(60).await;
        }

        let final_text = format!("{} [OK]", task);
        self.clear_line_at_y(current_y);
        self.draw_boot_line(&final_text, current_y, opts.color.as_deref());

        // Add to new line buffer
        line_buffer::add_line(final_text, LineType::Boot, opts.color.clone());

        self.advance_y();
        self.handle_scroll_if_needed();
    }

    async fn typing(&self, text: &str, speed: i32, opts: &LineOptions) {
        let current_y = self.y.get();
        let mut displayed = String::new();

        for ch in text.chars() {
            displayed.push(ch);
            self.clear_line_at_y(current_y);
            self.draw_text(&displayed, 10.0, current_y, opts.color.as_deref());
            self.sleep(speed).await;
        }

        // Add to new line buffer
        line_buffer::add_line(text.to_string(), LineType::Typing, opts.color.clone());

        self.advance_y();
        self.handle_scroll_if_needed();
    }

    async fn simple(&self, text: &str, opts: &LineOptions) {
        let current_y = self.y.get();

        // Add to new line buffer
        line_buffer::add_line(text.to_string(), LineType::Normal, opts.color.clone());

        self.draw_text(text, 10.0, current_y, opts.color.as_deref());
        self.advance_y();
        self.handle_scroll_if_needed();
    }

    /// Clear the entire canvas
    pub fn clear_screen(&self) {
        self.context.save();
        self.set_fill_color("#000000");
        self.context
            .fill_rect(0.0, 0.0, self.width as f64, self.height as f64);
        self.context.restore();
        self.y.set(20.0);
    }

    /// Calculate maximum lines that fit on screen
    pub fn max_visible_lines(&self) -> usize {
        ((self.height as f64 - 40.0) / self.line_height) as usize
    }

    /// Calculate maximum characters per line
    pub fn max_chars_per_line(&self) -> usize {
        ((self.width as f64 - 20.0) / self.char_width) as usize
    }

    /// Render the entire terminal state using the new line buffer
    pub fn render(&self) {
        self.clear_screen();

        // Update line buffer dimensions
        line_buffer::set_terminal_dimensions(self.max_chars_per_line(), self.max_visible_lines());

        // Get visible lines
        let visible_lines = line_buffer::get_visible_lines(self.max_visible_lines() - 2); // Reserve space for input
        let state = line_buffer::get_terminal_state();

        // Render lines
        let mut y_offset = 20.0;
        for line in visible_lines {
            y_offset += self.render_line(&line, y_offset);
        }

        // Update y position for legacy compatibility
        self.y.set(y_offset);

        // Render input line if in normal mode
        if state.input_mode == InputMode::Normal {
            self.render_input_line(&state, y_offset);
        }
    }

    /// Render a single buffer line
    fn render_line(&self, line: &BufferLine, y: f64) -> f64 {
        let color = self.get_color_for_line_type(&line.line_type, line.color.as_deref());

        if line.wrapped_lines.is_empty() {
            self.draw_text(&line.content, 10.0, y, Some(&color));
            self.line_height
        } else {
            let mut current_y = y;
            for wrapped_line in &line.wrapped_lines {
                self.draw_text(wrapped_line, 10.0, current_y, Some(&color));
                current_y += self.line_height;
            }
            self.line_height * line.wrapped_lines.len() as f64
        }
    }

    /// Render the input line with prompt and cursor
    fn render_input_line(&self, state: &TerminalState, y: f64) {
        // Clear the input line area
        self.clear_line_at_y(y);

        // Draw prompt
        let prompt_color = "#00ffff"; // Cyan
        self.draw_text(&state.prompt, 10.0, y, Some(prompt_color));

        // Draw input text
        let prompt_width = state.prompt.len() as f64 * self.char_width;
        let input_x = 10.0 + prompt_width;

        if !state.current_input.is_empty() {
            self.draw_text(&state.current_input, input_x, y, Some("#ffffff"));
        }

        // Draw cursor
        if self.cursor_blink_state.get() {
            let cursor_x = input_x + (state.cursor_position as f64 * self.char_width);
            self.draw_cursor(cursor_x, y);
        }
    }

    /// Legacy draw methods for compatibility
    pub fn draw_text(&self, text: &str, x: f64, y: f64, color: Option<&str>) {
        self.context.save();
        self.context.set_font("14px monospace");
        self.context.set_text_baseline("top");

        if let Some(color) = color {
            self.set_fill_color(&self.get_color_value(color));
        } else {
            self.set_fill_color("#ffffff");
        }

        let _ = self.context.fill_text(text, x, y);
        self.context.restore();
    }

    pub fn draw_boot_line(&self, text: &str, y: f64, color: Option<&str>) {
        self.context.save();

        self.context.set_font("14px monospace");
        self.context.set_text_baseline("top");

        if let Some(color) = color {
            self.set_fill_color(&self.get_color_value(color));
        } else {
            self.set_fill_color("#ffffff");
        }

        if let Some(ok_pos) = text.rfind(" [OK]") {
            let main_text = &text[..ok_pos];
            let ok_text = " [OK]";

            self.context.fill_text(main_text, 10.0, y).unwrap();

            let char_width = 8.4;
            let main_width = main_text.len() as f64 * char_width;

            self.set_fill_color("#00ff00");
            self.context
                .fill_text(ok_text, 10.0 + main_width, y)
                .unwrap();
        } else {
            self.context.fill_text(text, 10.0, y).unwrap();
        }

        self.context.restore();
    }

    fn clear_line_at_y(&self, y: f64) {
        self.context.save();
        self.set_fill_color("#000000");
        self.context
            .fill_rect(0.0, y, self.width as f64, self.line_height);
        self.context.restore();
    }

    /// Draw cursor
    fn draw_cursor(&self, x: f64, y: f64) {
        self.context.save();
        self.set_fill_color("#00ff00"); // Green cursor
        self.context.fill_rect(x, y, 2.0, self.line_height - 2.0);
        self.context.restore();
    }

    /// Set fill color for context
    fn set_fill_color(&self, color: &str) {
        let _ = js_sys::Reflect::set(
            &self.context,
            &JsValue::from_str("fillStyle"),
            &JsValue::from_str(color),
        );
    }

    /// Get color for line type
    fn get_color_for_line_type(&self, line_type: &LineType, custom_color: Option<&str>) -> String {
        if let Some(color) = custom_color {
            return self.get_color_value(color);
        }

        match line_type {
            LineType::Command => "#00ffff".to_string(), // Cyan for commands
            LineType::Output => "#ffffff".to_string(),  // White for output
            LineType::Error => "#ff0000".to_string(),   // Red for errors
            LineType::System => "#ffff00".to_string(),  // Yellow for system
            LineType::Boot => "#00ff00".to_string(),    // Green for boot
            LineType::Typing => "#ffffff".to_string(),  // White for typing
            LineType::Prompt => "#00ffff".to_string(),  // Cyan for prompts
            LineType::Normal => "#ffffff".to_string(),  // White for normal
        }
    }

    pub fn get_color_value(&self, color: &str) -> String {
        match color {
            "red" => "#ff0000".to_string(),
            "green" => "#00ff00".to_string(),
            "blue" => "#0000ff".to_string(),
            "yellow" => "#ffff00".to_string(),
            "cyan" => "#00ffff".to_string(),
            "magenta" => "#ff00ff".to_string(),
            "white" => "#ffffff".to_string(),
            "gray" | "grey" => "#808080".to_string(),
            "boot-line" => "#ffffff".to_string(),
            "typing-line" => "#ffffff".to_string(),
            "command" => "#8be9fd".to_string(),
            "completion" => "#f8f8f2".to_string(),
            "error" => "#ff4444".to_string(),
            "success" => "#44ff44".to_string(),
            "warning" => "#ffaa00".to_string(),
            _ => {
                if color.starts_with('#') || color.starts_with("rgb") {
                    color.to_string()
                } else {
                    "#ffffff".to_string()
                }
            }
        }
    }

    fn advance_y(&self) {
        let new_y = self.y.get() + self.line_height;
        self.y.set(new_y);
    }

    // Add method to handle automatic scrolling
    fn handle_scroll_if_needed(&self) {
        let max_lines = (self.height as f64 / self.line_height) as i32;
        let current_line = ((self.y.get() - 20.0) / self.line_height) as i32;

        if current_line >= max_lines - 3 {
            // Need to scroll - use new render system
            self.render();
        }
    }

    pub fn clear_output(&self) {
        line_buffer::clear_buffer();
        self.clear_screen();
        self.prepare_for_input();
    }

    /// Method to draw the current input line with prompt
    pub fn draw_current_input_line(&self, input: &str) {
        let state = line_buffer::get_terminal_state();
        let current_y = self.y.get();

        // Clear the current line first
        self.clear_line_at_y(current_y);

        // Draw prompt in cyan
        self.draw_text(&state.prompt, 10.0, current_y, Some("#00ffff"));

        // Draw input part
        if !input.is_empty() {
            let prompt_width = state.prompt.chars().count() as f64 * self.char_width;
            self.draw_text(input, 10.0 + prompt_width, current_y, Some("#ffffff"));
        }
    }

    /// Method to draw cursor at current position
    pub fn draw_cursor_legacy(&self, input: &str) {
        let state = line_buffer::get_terminal_state();
        let current_y = self.y.get();
        let prompt_width = state.prompt.chars().count() as f64 * self.char_width;
        let input_width = input.chars().count() as f64 * self.char_width;
        let cursor_x = 10.0 + prompt_width + input_width;

        self.context.save();
        self.set_fill_color("#00ff00"); // green cursor
        self.context.fill_text("█", cursor_x, current_y).unwrap();
        self.context.restore();
    }

    /// Method to prepare for input (ensures prompt is visible)
    pub fn prepare_for_input(&self) {
        // Set up terminal state
        let _cwd = "/home/objz"; // This should come from command handler
        let display_path = "~";
        let prompt = format!("objz@objz:{}$ ", display_path);

        line_buffer::set_current_prompt(prompt);
        line_buffer::set_input_mode(InputMode::Normal);
        line_buffer::update_input_state(String::new(), 0);
        line_buffer::auto_scroll_to_bottom();

        // Render the terminal
        self.render();
    }

    /// Method to update input display in real-time
    pub fn update_input_display(&self, input: &str) {
        let cursor_pos = input.len(); // For now, cursor is at end
        line_buffer::update_input_state(input.to_string(), cursor_pos);
        self.render();
    }

    /// Method to finalize input (when Enter is pressed)
    pub fn finalize_input(&self, input: &str) {
        let state = line_buffer::get_terminal_state();

        // Add the command to the buffer
        if !input.trim().is_empty() {
            line_buffer::add_command_line(&state.prompt, input);
        }

        // Set processing mode
        line_buffer::set_input_mode(InputMode::Processing);
        line_buffer::update_input_state(String::new(), 0);
    }

    /// Toggle cursor blink state
    pub fn toggle_cursor(&self) {
        self.cursor_blink_state.set(!self.cursor_blink_state.get());
    }

    /// Force cursor visible
    pub fn show_cursor(&self) {
        self.cursor_blink_state.set(true);
    }

    /// Force cursor hidden
    pub fn hide_cursor(&self) {
        self.cursor_blink_state.set(false);
    }

    pub async fn sleep(&self, ms: i32) {
        let promise = Promise::new(&mut |resolve, _reject| {
            let window = window().unwrap();
            let closure = wasm_bindgen::prelude::Closure::once_into_js(move || {
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

    pub fn get_current_prompt(&self) -> String {
        line_buffer::get_terminal_state().prompt
    }
}

impl Clone for TerminalRenderer {
    fn clone(&self) -> Self {
        Self {
            canvas: self.canvas.clone(),
            context: self.context.clone(),
            y: Cell::new(self.y.get()),
            width: self.width,
            height: self.height,
            line_height: self.line_height,
            char_width: self.char_width,
            font_size: self.font_size,
            cursor_blink_state: Cell::new(self.cursor_blink_state.get()),
        }
    }
}
