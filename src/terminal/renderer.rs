use crate::input::autoscroll::ensure_autoscroll;
use crate::terminal::Terminal;
use crate::utils::dom::append_line;

impl Terminal {
    pub async fn add_line_boot(&self, task: &str, status: &str, _color: &str) {
        let div = self.create_div_element("", Some("boot-line"));
        self.output_element.append_child(&div).unwrap();
        ensure_autoscroll(); // Add autoscroll immediately after adding element

        for i in 0..4 {
            let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let text = format!("{} {}", task, spinner[i]);
            div.set_inner_html(&text);
            ensure_autoscroll(); // Autoscroll after each spinner update
            self.sleep(60).await;
        }

        let final_html = if status.is_empty() {
            task.to_string()
        } else {
            format!("{} <span class=\"status\">{}</span>", task, status)
        };
        div.set_inner_html(&final_html);
        ensure_autoscroll(); // Final autoscroll
    }

    pub async fn add_line_typing(&self, text: &str, speed: u32) {
        let div = self.create_div_element("", None);
        div.set_class_name("typing-line");
        self.output_element.append_child(&div).unwrap();
        ensure_autoscroll(); // Add autoscroll immediately after adding element

        let mut buf = String::new();
        for (i, ch) in text.chars().enumerate() {
            buf.push(ch);
            let display = if i < text.len() - 1 {
                format!("{}<span class=\"typing-cursor\">█</span>", buf)
            } else {
                buf.clone()
            };
            div.set_inner_html(&display);
            ensure_autoscroll(); // Autoscroll after each character
            self.sleep(speed as i32).await;
        }
        div.set_inner_html(&buf);
        ensure_autoscroll(); // Final autoscroll
    }

    pub async fn add_colored_line_typing(&self, text: &str, speed: u32, color: &str) {
        let div = self.create_div_element("", Some(color));
        div.set_class_name(&format!("typing-line {}", color));
        self.output_element.append_child(&div).unwrap();
        ensure_autoscroll(); // Add autoscroll immediately after adding element

        let mut buf = String::new();
        for (i, ch) in text.chars().enumerate() {
            buf.push(ch);
            let display = if i < text.len() - 1 {
                format!("{}<span class=\"typing-cursor\">█</span>", buf)
            } else {
                buf.clone()
            };
            div.set_inner_html(&display);
            ensure_autoscroll(); // Autoscroll after each character
            self.sleep(speed as i32).await;
        }
        div.set_inner_html(&buf);
        ensure_autoscroll(); // Final autoscroll
    }

    pub async fn add_line(&self, text: &str) {
        append_line(&self.output_element, text, None);
        ensure_autoscroll(); // Autoscroll immediately after adding line
    }

    pub async fn add_line_colored(&self, text: &str, color: &str) {
        append_line(&self.output_element, text, Some(color));
        ensure_autoscroll(); // Autoscroll immediately after adding line
    }
}
