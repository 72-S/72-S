// Legacy compatibility module - keeping for backward compatibility
use crate::terminal::line_buffer;

pub use crate::terminal::line_buffer::{BufferLine as TerminalLine, LineType};

// Re-export new functions with old names for compatibility
pub fn add_line_to_buffer(text: String, color: Option<String>, line_type: LineType) {
    line_buffer::add_line(text, line_type, color);
}

pub fn clear_terminal_buffer() {
    line_buffer::clear_buffer();
}

pub fn get_terminal_lines() -> Vec<TerminalLine> {
    line_buffer::get_visible_lines(50) // Get reasonable number of lines
}

pub fn get_line_count() -> usize {
    line_buffer::LINE_BUFFER.with(|buffer| buffer.line_count())
}

pub fn trim_output(canvas_height: i32) {
    // This is now handled automatically by the line buffer system
    let line_height = 20.0;
    let max_lines = ((canvas_height as f64 - 40.0) / line_height) as usize;
    let max_chars = 80; // Default terminal width

    line_buffer::set_terminal_dimensions(max_chars, max_lines);
}

pub fn should_autoscroll() -> bool {
    line_buffer::LINE_BUFFER.with(|buffer| buffer.should_auto_scroll())
}

pub fn get_available_input_lines() -> usize {
    2 // Reserve 2 lines for input
}

// Convenience functions for compatibility
pub fn add_simple_line(text: String, color: Option<String>) {
    line_buffer::add_line(text, LineType::Normal, color);
}
