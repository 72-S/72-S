use crate::input::autoscroll::{ensure_autoscroll, trim_output};
use crate::terminal::Terminal;
use crate::utils::dom::append_line;

impl Terminal {
    pub async fn add_line_boot(&self, task: &str, status: &str, _color: &str) {
        let div = self.create_div_element("", Some("boot-line"));
        self.output_element.append_child(&div).unwrap();
        ensure_autoscroll();

        for i in 0..4 {
            let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let text = format!("{} {}", task, spinner[i]);
            div.set_inner_html(&text);
            ensure_autoscroll();
            self.sleep(60).await;
        }

        let final_html = if status.is_empty() {
            task.to_string()
        } else {
            format!("{} <span class=\"status\">{}</span>", task, status)
        };
        div.set_inner_html(&final_html);
        trim_output(self.height);
        ensure_autoscroll();
    }

    pub async fn add_line_typing(&self, text: &str, speed: u32) {
        let div = self.create_div_element("", None);
        div.set_class_name("typing-line");
        self.output_element.append_child(&div).unwrap();
        ensure_autoscroll();

        let mut buf = String::new();
        for (i, ch) in text.chars().enumerate() {
            buf.push(ch);
            let display = if i < text.len() - 1 {
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

    pub async fn add_colored_line_typing(&self, text: &str, speed: u32, color: &str) {
        let div = self.create_div_element("", Some(color));
        div.set_class_name(&format!("typing-line {}", color));
        self.output_element.append_child(&div).unwrap();
        ensure_autoscroll();

        let mut buf = String::new();
        for (i, ch) in text.chars().enumerate() {
            buf.push(ch);
            let display = if i < text.len() - 1 {
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

    pub async fn add_line(&self, text: &str) {
        append_line(&self.output_element, text, None);
        ensure_autoscroll();
    }

    pub async fn add_line_colored(&self, text: &str, color: &str) {
        append_line(&self.output_element, text, Some(color));
        ensure_autoscroll();
    }
}
