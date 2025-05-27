use crate::terminal::Terminal;

pub mod boot;

impl Terminal {
    pub async fn run_boot_sequence(&self) {
        self.clear_output();
        boot::boot(self).await;
        boot::logo(self).await;
        boot::login(self).await;
    }
}
