use std::cell::RefCell;
use std::collections::VecDeque;

// Terminal line with its properties
#[derive(Clone)]
pub struct TerminalLine {
    pub text: String,
    pub color: Option<String>,
    pub line_type: LineType,
}

#[derive(Clone)]
pub enum LineType {
    Normal,
    Boot,
    Typing,
    Prompt,
    Input, // Add Input type for current input line
}

// Store terminal lines for scrolling
thread_local! {
    static TERMINAL_LINES: RefCell<VecDeque<TerminalLine>> = RefCell::new(VecDeque::new());
    static MAX_LINES: RefCell<usize> = RefCell::new(25);
    static RESERVE_LINES: RefCell<usize> = RefCell::new(2); // Reserve space for input
}

pub fn trim_output(canvas_height: i32) {
    let line_height = 20.0;
    let total_lines = ((canvas_height as f64 - 40.0) / line_height) as usize;

    // Reserve space for input line
    RESERVE_LINES.with(|reserve| {
        let reserve_count = *reserve.borrow();
        let max_visible_lines = if total_lines > reserve_count {
            total_lines - reserve_count
        } else {
            1
        };

        MAX_LINES.with(|max| {
            *max.borrow_mut() = max_visible_lines;
        });
    });

    TERMINAL_LINES.with(|lines| {
        let mut lines_mut = lines.borrow_mut();
        MAX_LINES.with(|max| {
            let max_lines = *max.borrow();
            while lines_mut.len() > max_lines {
                lines_mut.pop_front();
            }
        });
    });
}

pub fn add_line_to_buffer(text: String, color: Option<String>, line_type: LineType) {
    TERMINAL_LINES.with(|lines| {
        let mut lines_mut = lines.borrow_mut();

        lines_mut.push_back(TerminalLine {
            text,
            color,
            line_type,
        });

        MAX_LINES.with(|max| {
            let max_lines = *max.borrow();
            while lines_mut.len() > max_lines {
                lines_mut.pop_front();
            }
        });
    });
}

// Convenience function for simple lines (keeping 2-parameter version for compatibility)
pub fn add_simple_line(text: String, color: Option<String>) {
    add_line_to_buffer(text, color, LineType::Normal);
}

pub fn get_terminal_lines() -> Vec<TerminalLine> {
    TERMINAL_LINES.with(|lines| lines.borrow().iter().cloned().collect())
}

pub fn clear_terminal_buffer() {
    TERMINAL_LINES.with(|lines| {
        lines.borrow_mut().clear();
    });
}

pub fn get_line_count() -> usize {
    TERMINAL_LINES.with(|lines| lines.borrow().len())
}

// Function to check if autoscrolling is needed
pub fn should_autoscroll() -> bool {
    MAX_LINES.with(|max| {
        TERMINAL_LINES.with(|lines| {
            let line_count = lines.borrow().len();
            let max_lines = *max.borrow();
            line_count >= max_lines
        })
    })
}

pub fn get_available_input_lines() -> usize {
    RESERVE_LINES.with(|reserve| *reserve.borrow())
}
