use std::cell::Cell;

use crate::commands::CommandHandler;
use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, CanvasRenderingContext2d, Document, HtmlCanvasElement};

#[derive(Clone)]
pub struct Terminal {
    pub canvas: HtmlCanvasElement,
    pub context: CanvasRenderingContext2d,
    pub y: Cell<f64>,
    pub line_height: f64,
    pub command_handler: CommandHandler,
    pub base_prompt: String,
    pub height: i32,
    pub width: i32,
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

        // Set up canvas context with proper font settings
        context.set_font("14px monospace");
        context.set_text_baseline("top");

        // Use fill_style property directly (newer non-deprecated API)
        let _ = js_sys::Reflect::set(
            &context,
            &JsValue::from_str("fillStyle"),
            &JsValue::from_str("#ffffff"),
        );

        // Enable crisp pixel rendering
        context.set_image_smoothing_enabled(false);

        let command_handler = CommandHandler::new();
        let base_prompt = "objz@objz".to_string();
        let width = canvas_width as i32;
        let height = canvas_height as i32;

        // Clear canvas initially
        context.save();
        let _ = js_sys::Reflect::set(
            &context,
            &JsValue::from_str("fillStyle"),
            &JsValue::from_str("#000000"),
        );
        context.fill_rect(0.0, 0.0, width as f64, height as f64);
        context.restore();

        Self {
            canvas,
            context,
            y: Cell::new(20.0),
            line_height: 20.0,
            command_handler,
            base_prompt,
            height,
            width,
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
}
