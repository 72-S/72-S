use crate::commands::{filesystem, system};

#[derive(Clone)]
pub struct CommandHandler {
    history: Vec<String>,
}

impl CommandHandler {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }

    pub fn get_current_directory(&self) -> String {
        filesystem::pwd(&[])
    }

    pub fn handle(&mut self, input: &str) -> (String, bool) {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return (String::new(), false);
        }

        self.history.push(trimmed.to_string());
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let cmd = parts[0];
        let args = &parts[1..];

        let directory_changed = cmd == "cd";

        let output = match cmd {
            "clear" => system::clear(args),
            "history" => self.show_history(args),
            "echo" => system::echo(args),
            "date" => system::date(args),
            "uptime" => system::uptime(args),
            "neofetch" => system::neofetch(args),

            "ls" => filesystem::ls(args),
            "cd" => filesystem::cd(args),
            "cat" => filesystem::cat(args),
            "pwd" => filesystem::pwd(args),
            "tree" => filesystem::tree(args),
            "mkdir" => filesystem::mkdir(args),
            "touch" => filesystem::touch(args),
            "rm" => filesystem::rm(args),
            "uname" => filesystem::uname(args),
            "ln" => filesystem::ln(args),
            "ll" => filesystem::ls(&["-la"]),

            _ => format!("zsh: {}: command not found", cmd),
        };

        (output, directory_changed)
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
