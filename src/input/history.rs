use std::collections::VecDeque;

#[derive(Clone)]
pub struct CommandHistory {
    commands: VecDeque<String>,
    current_index: Option<usize>,
    max_size: usize,
}

impl CommandHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            commands: VecDeque::new(),
            current_index: None,
            max_size,
        }
    }

    pub fn add(&mut self, cmd: String) {
        let trimmed = cmd.trim();
        if !trimmed.is_empty() {
            if let Some(pos) = self.commands.iter().position(|x| x == trimmed) {
                self.commands.remove(pos);
            }
            self.commands.push_front(trimmed.to_string());
            if self.commands.len() > self.max_size {
                self.commands.pop_back();
            }
        }
        self.current_index = None;
    }

    pub fn previous(&mut self) -> Option<String> {
        if self.commands.is_empty() {
            return None;
        }
        match self.current_index {
            None => {
                self.current_index = Some(0);
                self.commands.get(0).cloned()
            }
            Some(idx) if idx + 1 < self.commands.len() => {
                self.current_index = Some(idx + 1);
                self.commands.get(idx + 1).cloned()
            }
            Some(idx) => self.commands.get(idx).cloned(),
        }
    }

    pub fn next(&mut self) -> Option<String> {
        match self.current_index {
            None => None,
            Some(0) => {
                self.current_index = None;
                Some(String::new())
            }
            Some(idx) => {
                let new_idx = idx - 1;
                self.current_index = Some(new_idx);
                self.commands.get(new_idx).cloned()
            }
        }
    }
}
