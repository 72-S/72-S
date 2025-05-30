use crate::input::autoscroll::{ensure_autoscroll, trim_output};
use crate::terminal::Terminal;
use web_sys::{window, Element};

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
        let class_name = if let Some(ref color) = opts.color {
            format!("boot-line {}", color)
        } else {
            "boot-line".to_string()
        };

        let div = self.create_div("", Some(&class_name));
        self.canvas.append_child(&div).unwrap();
        ensure_autoscroll();

        let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        for &spin_char in &spinner[0..4] {
            let text = format!("{} {}", task, spin_char);
            div.set_inner_html(&text);
            ensure_autoscroll();
            self.sleep(60).await;
        }

        // Set final content
        let final_html = if let Some(ref color) = opts.color {
            format!("{} <span class=\"status {}\">[OK]</span>", task, color)
        } else {
            format!("{} <span class=\"status\">[OK]</span>", task)
        };

        div.set_inner_html(&final_html);
        trim_output(self.height);
        ensure_autoscroll();
    }

    async fn typing(&self, text: &str, speed: u32, opts: &LineOptions) {
        let class_name = if let Some(ref color) = opts.color {
            format!("typing-line {}", color)
        } else {
            "typing-line".to_string()
        };

        let div = self.create_div("", Some(&class_name));
        self.canvas.append_child(&div).unwrap();
        ensure_autoscroll();

        let mut buf = String::new();
        let chars: Vec<char> = text.chars().collect();

        for (i, &ch) in chars.iter().enumerate() {
            buf.push(ch);
            let display = if i < chars.len() - 1 {
                format!("{}<span class=\"typing-cursor\">█</span>", buf)
            } else {
                buf.clone()
            };
            div.set_inner_html(&display);
            ensure_autoscroll();
            self.sleep(speed as i32).await;
        }

        div.set_inner_html(&buf);
        trim_output(self.height);
        ensure_autoscroll();
    }

    async fn simple(&self, text: &str, opts: &LineOptions) {
        self.append_line(text, opts.color.as_deref());
        ensure_autoscroll();
    }

    fn create_div(&self, text: &str, class: Option<&str>) -> Element {
        let div = window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("div")
            .unwrap();

        if text.is_empty() {
            div.set_inner_html("");
        } else {
            div.set_text_content(Some(text));
        }

        if let Some(class) = class {
            div.set_class_name(class);
        }

        div
    }

    fn append_line(&self, text: &str, class: Option<&str>) {
        let div = self.create_div(text, class);
        self.canvas.append_child(&div).unwrap();
        trim_output(self.height);
        ensure_autoscroll();
    }

    pub fn clear_output(&self) {
        self.canvas.set_inner_html("");
    }
}
