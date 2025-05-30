use crate::input::autoscroll::{ensure_autoscroll, trim_output};
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
            ensure_autoscroll();
            self.sleep(60).await;
        }

        let final_text = format!("{} [OK]", task);
        self.clear_line_at_y(current_y);
        self.draw_boot_line(&final_text, current_y, opts.color.as_deref());

        self.advance_y();
        trim_output(self.height);
        ensure_autoscroll();
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
            ensure_autoscroll();
            self.sleep(speed as i32).await;
        }

        self.clear_line_at_y(current_y);
        self.draw_text(&buf, current_y, opts.color.as_deref());

        self.advance_y();
        trim_output(self.height);
        ensure_autoscroll();
    }

    async fn simple(&self, text: &str, opts: &LineOptions) {
        self.append_line(text, opts.color.as_deref());
        ensure_autoscroll();
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
            let text_without_cursor = &text[..text.len() - 1];
            let cursor = "█";

            self.context
                .fill_text(text_without_cursor, 10.0, y)
                .unwrap();

            let char_width = 8.4;
            let text_width = text_without_cursor.len() as f64 * char_width;

            self.set_fill_color("#00ff00");
            self.context
                .fill_text(cursor, 10.0 + text_width, y)
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

    fn get_color_value(&self, color: &str) -> String {
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
        self.draw_text(text, current_y, color);
        self.advance_y();
        trim_output(self.height);
    }

    pub fn clear_output(&self) {
        self.context.save();
        self.set_fill_color("#000000");
        self.context
            .fill_rect(0.0, 0.0, self.width as f64, self.height as f64);
        self.context.restore();

        self.y.set(20.0);
    }
}
