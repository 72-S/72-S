use crate::terminal::Terminal;

impl Terminal {
    pub async fn run_boot_sequence(&self) {
        self.clear_output();
        self.run_system_boot().await;
        self.show_logo().await;
        self.run_login_sequence().await;
        self.run_portfolio_initialization().await;
        self.show_welcome_message().await;
    }

    async fn run_system_boot(&self) {
        let boot_messages = vec![
            ("Loading Linux linux...", "[OK]", "green"),
            ("Loading initial ramdisk...", "[OK]", "green"),
            (
                "Starting systemd-udevd version 254.5-1-arch",
                "[OK]",
                "green",
            ),
            ("Found device /dev/sda1", "[OK]", "green"),
            ("Started systemd-fsck@dev-disk-by\\x2duuid", "[OK]", "green"),
            ("Mounting /boot...", "[OK]", "green"),
            ("Mounting /home...", "[OK]", "green"),
            ("Starting Create System Users...", "[OK]", "green"),
            (
                "Starting systemd-tmpfiles-setup-dev.service",
                "[OK]",
                "green",
            ),
            ("Starting Load Kernel Modules...", "[OK]", "green"),
            ("Started systemd-journald.service", "[OK]", "green"),
            (
                "Starting Flush Journal to Persistent Storage...",
                "[OK]",
                "green",
            ),
            ("Starting Network Time Synchronization...", "[OK]", "green"),
            ("Starting User Login Management...", "[OK]", "green"),
        ];

        for (message, status, color) in boot_messages {
            self.add_line_with_boot_animation(message, status, color)
                .await;
            self.sleep(50).await; // Reduced from 150ms
        }

        self.add_line("").await;
        self.add_line_with_boot_animation("Starting objz Portfolio System", "[OK]", "green")
            .await;
        self.sleep(200).await; // Reduced from 300ms
        self.add_line("").await;
    }

    async fn show_logo(&self) {
        let logo_lines = vec![
            "  ___  ___ ___  ____",
            " / _ \\| _ ) _ \\|_  /",
            "| (_) | _ \\   / / / ",
            " \\___/|___/_|_\\/___|",
            "",
        ];

        for line in logo_lines {
            if line.starts_with(" ")
                && (line.contains("/") || line.contains("\\") || line.contains("|"))
            {
                self.add_line_with_typing_color(line, 30, "cyan").await; // Smooth typing with cyan
            } else {
                self.add_line_instant(line).await;
            }
            self.sleep(40).await; // Reduced from 80ms
        }
    }

    async fn run_login_sequence(&self) {
        let login_messages = vec![
            ("Arch Linux 6.6.32-1-lts (tty1)", "", "green"),
            ("", "", ""),
            ("objz-portfolio login: objz", "", "white"),
            ("Password: ", "", "white"),
            ("", "", ""),
            ("Last login: Mon May 27 13:59:36 2024", "", "white"),
            ("", "", ""),
        ];

        for (message, _status, color) in login_messages {
            if message.is_empty() {
                // blank line
                self.add_line("").await;
            } else if message.contains("login:") {
                // type the entire login prompt + username on one line
                self.add_line_with_typing(message, 50).await;
            } else if message.contains("Password:") {
                // type the prompt and dots on one line
                let full = format!("{}••••••••", message);
                self.add_line_with_typing(&full, 50).await;
            } else {
                // all other static lines
                self.add_line_instant_with_color(message, color).await;
            }
            self.sleep(60).await; // preserve your existing pacing
        }
    }

    async fn run_portfolio_initialization(&self) {
        let init_messages = vec![
            ("Starting portfolio services", "[OK]", "green"),
            ("Loading project database", "[OK]", "green"),
            ("Initializing web terminal", "[OK]", "green"),
            ("Setting up command processor", "[OK]", "green"),
            ("Establishing connections", "[OK]", "green"),
        ];

        for (message, status, color) in init_messages {
            self.add_line_with_boot_animation(message, status, color)
                .await;
            self.sleep(60).await; // Reduced from 100ms
        }
    }

    async fn show_welcome_message(&self) {
        self.add_line("").await;
        self.add_line_with_typing_color("Welcome to objz's Portfolio Terminal", 40, "green")
            .await;
        self.sleep(300).await;
        self.add_line_with_typing("Type 'help' to see available commands", 30)
            .await;
        self.add_line("").await;
    }
}
