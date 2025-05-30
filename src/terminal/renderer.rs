use crate::input::autoscroll::{add_line_to_buffer, trim_output, LineType};
use crate::terminal::Terminal;
use wasm_bindgen::JsValue;

#[derive(Default)]
pub struct LineOptions {
    pub color: Option<String>,
    pub typing_speed: Option<u32>,
    pub boot_animation: bool,
}

impl LineOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_color(mut self, color: &str) -> Self {
        self.color = Some(color.to_string());
        self
    }

    pub fn with_typing(mut self, speed: u32) -> Self {
        self.typing_speed = Some(speed);
        self
    }

    pub fn with_boot_animation(mut self) -> Self {
        self.boot_animation = true;
        self
    }
}

impl Terminal {
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
            self.draw_text(&text, current_y, opts.color.as_deref());
            self.sleep(60).await;
        }

        let final_text = format!("{} [OK]", task);
        self.clear_line_at_y(current_y);
        self.draw_boot_line(&final_text, current_y, opts.color.as_deref());

        // Add the final line to the buffer for scrolling (with LineType::Boot)
        add_line_to_buffer(final_text, opts.color.clone(), LineType::Boot);

        self.advance_y();
        trim_output(self.height);
        self.handle_scroll_if_needed();
    }

    async fn typing(&self, text: &str, speed: u32, opts: &LineOptions) {
        let current_y = self.y.get();

        let mut buf = String::new();
        let chars: Vec<char> = text.chars().collect();

        for (i, &ch) in chars.iter().enumerate() {
            buf.push(ch);
            self.clear_line_at_y(current_y);

            let display = if i < chars.len() - 1 {
                format!("{}█", buf)
            } else {
                buf.clone()
            };

            self.draw_typing_text(
                &display,
                current_y,
                opts.color.as_deref(),
                i < chars.len() - 1,
            );
            self.sleep(speed as i32).await;
        }

        self.clear_line_at_y(current_y);
        self.draw_text(&buf, current_y, opts.color.as_deref());

        // Add the final typed line to the buffer (with LineType::Typing)
        add_line_to_buffer(buf, opts.color.clone(), LineType::Typing);

        self.advance_y();
        trim_output(self.height);
        self.handle_scroll_if_needed();
    }

    async fn simple(&self, text: &str, opts: &LineOptions) {
        self.append_line(text, opts.color.as_deref());
    }

    fn set_fill_color(&self, color: &str) {
        let _ = js_sys::Reflect::set(
            &self.context,
            &JsValue::from_str("fillStyle"),
            &JsValue::from_str(color),
        );
    }

    fn draw_text(&self, text: &str, y: f64, color: Option<&str>) {
        self.context.save();

        self.context.set_font("14px monospace");
        self.context.set_text_baseline("top");

        if let Some(color) = color {
            self.set_fill_color(&self.get_color_value(color));
        } else {
            self.set_fill_color("#ffffff");
        }

        self.context.fill_text(text, 10.0, y).unwrap();
        self.context.restore();
    }

    fn draw_boot_line(&self, text: &str, y: f64, color: Option<&str>) {
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

    fn draw_typing_text(&self, text: &str, y: f64, color: Option<&str>, has_cursor: bool) {
        self.context.save();

        self.context.set_font("14px monospace");
        self.context.set_text_baseline("top");

        if let Some(color) = color {
            self.set_fill_color(&self.get_color_value(color));
        } else {
            self.set_fill_color("#ffffff");
        }

        if has_cursor {
            // Fixed: Handle Unicode characters properly
            let chars: Vec<char> = text.chars().collect();
            if chars.len() > 0 && chars[chars.len() - 1] == '█' {
                // Remove the cursor character properly
                let text_without_cursor: String = chars[..chars.len() - 1].iter().collect();
                let cursor = "█";

                self.context
                    .fill_text(&text_without_cursor, 10.0, y)
                    .unwrap();

                let char_width = 8.4;
                let text_width = text_without_cursor.chars().count() as f64 * char_width;

                self.set_fill_color("#00ff00");
                self.context
                    .fill_text(cursor, 10.0 + text_width, y)
                    .unwrap();
            } else {
                // Fallback: just draw the text as-is
                self.context.fill_text(text, 10.0, y).unwrap();
            }
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

    fn append_line(&self, text: &str, color: Option<&str>) {
        let current_y = self.y.get();

        // Add to buffer with LineType::Normal
        add_line_to_buffer(
            text.to_string(),
            color.map(|c| c.to_string()),
            LineType::Normal,
        );

        self.draw_text(text, current_y, color);
        self.advance_y();
        trim_output(self.height);
        self.handle_scroll_if_needed();
    }

    // Add method to handle automatic scrolling
    fn handle_scroll_if_needed(&self) {
        let max_lines = (self.height as f64 / self.line_height) as i32;
        let current_line = ((self.y.get() - 20.0) / self.line_height) as i32;

        if current_line >= max_lines - 3 {
            // Leave space for input
            // Need to scroll - redraw all buffered lines
            self.redraw_all_lines();
        }
    }

    // Method to redraw all lines from buffer (for scrolling)
    fn redraw_all_lines(&self) {
        // Clear canvas
        self.clear_output_no_buffer_clear();

        // Get all lines from buffer
        let lines = crate::input::autoscroll::get_terminal_lines();

        // Reset Y position
        self.y.set(20.0);

        // Redraw each line
        for line in lines {
            let current_y = self.y.get();

            match line.line_type {
                LineType::Boot => {
                    self.draw_boot_line(&line.text, current_y, line.color.as_deref());
                }
                LineType::Typing => {
                    self.draw_text(&line.text, current_y, line.color.as_deref());
                }
                LineType::Normal => {
                    self.draw_text(&line.text, current_y, line.color.as_deref());
                }
                LineType::Prompt => {
                    self.draw_text(&line.text, current_y, line.color.as_deref());
                }
                LineType::Input => {
                    self.draw_text(&line.text, current_y, line.color.as_deref());
                }
            }

            self.advance_y();
        }
    }

    // Clear output without clearing buffer (for internal use)
    fn clear_output_no_buffer_clear(&self) {
        self.context.save();
        self.set_fill_color("#000000");
        self.context
            .fill_rect(0.0, 0.0, self.width as f64, self.height as f64);
        self.context.restore();

        self.y.set(20.0);
    }

    pub fn clear_output(&self) {
        self.context.save();
        self.set_fill_color("#000000");
        self.context
            .fill_rect(0.0, 0.0, self.width as f64, self.height as f64);
        self.context.restore();

        self.y.set(20.0);

        crate::input::autoscroll::clear_terminal_buffer();

        // After clearing, immediately prepare for input
        self.prepare_for_input();
    }

    // NEW METHODS FOR CANVAS-BASED INPUT

    // Method to draw the current input line with prompt
    pub fn draw_current_input_line(&self, input: &str) {
        let current_y = self.y.get();
        let prompt = self.get_current_prompt();

        // Clear the current line first
        self.clear_line_at_y(current_y);

        // Draw prompt in cyan
        self.context.save();
        self.context.set_font("14px monospace");
        self.context.set_text_baseline("top");

        // Draw prompt part
        self.set_fill_color("#00ffff"); // cyan
        self.context.fill_text(&prompt, 10.0, current_y).unwrap();

        // Draw input part
        if !input.is_empty() {
            let char_width = 8.4;
            let prompt_width = prompt.chars().count() as f64 * char_width;
            self.set_fill_color("#ffffff"); // white for input
            self.context
                .fill_text(input, 10.0 + prompt_width, current_y)
                .unwrap();
        }

        self.context.restore();
    }

    // Method to draw cursor at current position
    pub fn draw_cursor(&self, input: &str) {
        let current_y = self.y.get();
        let prompt = self.get_current_prompt();
        let char_width = 8.4;
        let prompt_width = prompt.chars().count() as f64 * char_width;
        let input_width = input.chars().count() as f64 * char_width;
        let cursor_x = 10.0 + prompt_width + input_width;

        self.context.save();
        self.set_fill_color("#00ff00"); // green cursor
        self.context.fill_text("█", cursor_x, current_y).unwrap();
        self.context.restore();
    }

    // Method to prepare for input (ensures prompt is visible)
    pub fn prepare_for_input(&self) {
        // Make sure we have space for input
        let max_lines = (self.height as f64 / self.line_height) as i32;
        let current_line = ((self.y.get() - 20.0) / self.line_height) as i32;

        // If we're too close to the bottom, scroll
        if current_line >= max_lines - 2 {
            self.redraw_all_lines();
            // Position for new input line
            let lines_count = crate::input::autoscroll::get_line_count();
            let new_y = 20.0 + (lines_count as f64 * self.line_height);
            self.y.set(new_y);
        }

        // Draw initial prompt
        self.draw_current_input_line("");
        self.draw_cursor("");
    }

    // Method to update input display in real-time
    pub fn update_input_display(&self, input: &str) {
        self.draw_current_input_line(input);
        self.draw_cursor(input);
    }

    // Method to finalize input (when Enter is pressed) - FIXED
    pub fn finalize_input(&self, input: &str) {
        let prompt = self.get_current_prompt();
        let full_line = format!("{}{}", prompt, input);

        // Clear the current input line display
        let current_y = self.y.get();
        self.clear_line_at_y(current_y);

        // Add the complete line to buffer and draw it properly
        add_line_to_buffer(
            full_line.clone(),
            Some("white".to_string()),
            LineType::Normal,
        );
        self.draw_text(&full_line, current_y, Some("white"));

        // Move to next line
        self.advance_y();

        // Handle scrolling if needed
        self.handle_scroll_if_needed();
    }

    // Add method to add a prompt line
    pub fn add_prompt_line(&self, prompt: &str) {
        let current_y = self.y.get();

        // Add to buffer with LineType::Prompt
        add_line_to_buffer(
            prompt.to_string(),
            Some("cyan".to_string()),
            LineType::Prompt,
        );

        self.draw_text(prompt, current_y, Some("cyan"));
        self.advance_y();
        trim_output(self.height);
        self.handle_scroll_if_needed();
    }
}
