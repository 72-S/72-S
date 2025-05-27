use web_sys::{window, Element};

impl super::Terminal {
    pub fn create_div_element(&self, text: &str, class: Option<&str>) -> Element {
        let doc = window().unwrap().document().unwrap();
        let div = doc.create_element("div").unwrap();
        if text.is_empty() {
            div.set_inner_html("");
        } else {
            div.set_text_content(Some(text));
        }
        if let Some(cls) = class {
            div.set_class_name(cls);
        }
        div
    }

    pub fn clear_output(&self) {
        self.output_element.set_inner_html("");
    }
}
