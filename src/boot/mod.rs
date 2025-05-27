use crate::terminal::Terminal;

pub mod init;
pub mod login;
pub mod logo;
pub mod system;

impl Terminal {
    pub async fn run_boot_sequence(&self) {
        self.clear_output();
        system::run_system_boot(self).await;
        logo::show_logo(self).await;
        login::run_login_sequence(self).await;
        init::run_boot_init(self).await;
    }
}
