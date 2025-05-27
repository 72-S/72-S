use crate::ascii_art::AsciiArt;
use crate::commands::{filesystem, misc, network, system};

#[derive(Clone)]
pub struct CommandHandler {
    history: Vec<String>,
    ascii_art: AsciiArt,
}

impl CommandHandler {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            ascii_art: AsciiArt::new(),
        }
    }

    pub fn handle(&mut self, input: &str) -> String {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        self.history.push(trimmed.to_string());
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let cmd = parts[0];
        let args = &parts[1..];

        match cmd {
            // System commands
            "help" => system::help(args),
            "whoami" => system::whoami(args),
            "date" => system::date(args),
            "uptime" => system::uptime(args),
            "top" => system::top(args),
            "ps" => system::ps(args),
            "neofetch" => system::neofetch(args),
            "clear" => system::clear(args),

            // Filesystem commands
            "ls" => filesystem::list(args),
            "cat" => filesystem::display(args),

            // ASCII art
            "ascii" => self.ascii_art.get_ascii(args),
            "matrix" => self.ascii_art.get_matrix_effect(),

            // Network commands
            "telnet" => network::telnet(args),
            "nc" | "netcat" => network::netcat(args),

            // Miscellaneous utilities
            "echo" => misc::echo(args),
            "sudo" => misc::sudo(args),
            "make" => misc::make(args),
            "hack" => misc::hack(args),

            // History
            "history" => self.show_history(args),

            _ => format!("bash: {}: command not found", cmd),
        }
    }

    fn show_history(&self, _args: &[&str]) -> String {
        if self.history.is_empty() {
            "No commands in history yet.".to_string()
        } else {
            self.history
                .iter()
                .enumerate()
                .map(|(i, cmd)| format!("  {}  {}", i + 1, cmd))
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}
