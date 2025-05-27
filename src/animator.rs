#[derive(Clone)]
pub struct Animator;

impl Animator {
    pub fn new() -> Self {
        Self
    }

    pub fn get_loading_bar(&self, progress: u8) -> String {
        let filled = (progress as f32 / 100.0 * 20.0) as usize;
        let empty = 20 - filled;

        format!(
            "[{}{}] {}%",
            "█".repeat(filled),
            "░".repeat(empty),
            progress
        )
    }
}
