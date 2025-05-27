pub mod effects;
pub mod projects;
pub mod user;

use effects::get_matrix_effect;
use projects::get_project_ascii;
use user::get_user_ascii;

#[derive(Clone)]
pub struct AsciiArt;

impl AsciiArt {
    pub fn new() -> Self {
        Self
    }

    pub fn get_ascii(&self, args: &[&str]) -> String {
        if args.is_empty() {
            let topics = projects::AVAILABLE_TOPICS.join(", ");
            return format!("Usage: ascii <topic>\nAvailable topics: {}", topics);
        }
        get_project_ascii(args[0])
    }

    pub fn get_matrix_effect(&self) -> String {
        get_matrix_effect()
    }

    pub fn get_user_ascii() -> String {
        get_user_ascii()
    }
}
