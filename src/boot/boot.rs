use crate::terminal::{renderer::LineOptions, Terminal};

pub async fn boot(term: &Terminal) {
    let boot_messages = vec![
        (
            "Loading Linux kernel version 6.8.9-arch1-1...",
            "[OK]",
            "green",
        ),
        ("Loading initial ramdisk (initramfs)...", "[OK]", "green"),
        ("Starting systemd-udevd v254.5-1...", "[OK]", "green"),
        ("Probing hardware...", "[OK]", "green"),
        ("Detected storage device: /dev/nvme0n1", "[OK]", "green"),
        ("Detected storage device: /dev/sda", "[OK]", "green"),
        ("Started udev Kernel Device Manager.", "[OK]", "green"),
        ("Activating swap on /dev/sda2...", "[OK]", "green"),
        ("Mounting root filesystem...", "[OK]", "green"),
        ("Checking file system on /dev/sda1...", "[OK]", "green"),
        ("Mounting /boot...", "[OK]", "green"),
        ("Mounting /home...", "[OK]", "green"),
        ("Mounting /var...", "[OK]", "green"),
        ("Starting systemd-journald.service...", "[OK]", "green"),
        (
            "Starting systemd-tmpfiles-setup-dev.service...",
            "[OK]",
            "green",
        ),
        ("Starting systemd-sysctl.service...", "[OK]", "green"),
        ("Starting Load Kernel Modules...", "[OK]", "green"),
        ("Loading kernel modules: i915 ext4 fuse...", "[OK]", "green"),
        (
            "Started Rule-based Manager for Device Events and Filesystems.",
            "[OK]",
            "green",
        ),
        ("Starting Network Manager...", "[OK]", "green"),
        ("Started Network Time Synchronization.", "[OK]", "green"),
        (
            "Starting Login Service (systemd-logind)...",
            "[OK]",
            "green",
        ),
        (
            "Starting Authorization Manager (polkitd)...",
            "[OK]",
            "green",
        ),
        ("Starting User Manager for UID 1000...", "[OK]", "green"),
        ("Started Getty on tty1.", "[OK]", "green"),
        ("Reached target Multi-User System.", "[OK]", "green"),
        ("Starting Interface...", "[OK]", "green"),
    ];

    for (msg, _status, color) in boot_messages {
        term.add_line(
            msg,
            Some(LineOptions::new().with_boot_animation().with_color(color)),
        )
        .await;
        term.sleep(15).await;
    }
    term.add_line("", None).await;
    term.add_line(
        "Started objz Terminal",
        Some(LineOptions::new().with_boot_animation().with_color("green")),
    )
    .await;
    term.sleep(200).await;
    term.add_line("", None).await;
}

pub async fn logo(term: &Terminal) {
    let logo_lines = vec![
        "                                                    ",
        " ░▒▓██████▓▒░░▒▓███████▓▒░       ░▒▓█▓▒░▒▓████████▓▒░ ",
        "░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░      ░▒▓█▓▒░      ░▒▓█▓▒░",
        "░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░      ░▒▓█▓▒░    ░▒▓██▓▒░ ",
        "░▒▓█▓▒░░▒▓█▓▒░▒▓███████▓▒░       ░▒▓█▓▒░  ░▒▓██▓▒░   ",
        "░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░░▒▓██▓▒░     ",
        "░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░       ",
        " ░▒▓██████▓▒░░▒▓███████▓▒░ ░▒▓██████▓▒░░▒▓████████▓▒░",
        "                                                    ",
    ];

    for line in logo_lines {
        term.add_line(
            line,
            Some(LineOptions::new().with_typing(10).with_color("cyan")),
        )
        .await;
        term.sleep(30).await;
    }
}

pub async fn login(term: &Terminal) {
    let login_messages = vec![
        ("Arch Linux 6.6.32-1-lts (tty1)", "", "green"),
        ("", "", ""),
        ("login: anonym", "", "white"),
        ("password: ", "", "white"),
        ("", "", ""),
        ("Last login: Mon May 27 13:59:36 2025", "", "white"),
        ("Type 'help' for further information", "", "yellow"),
        ("", "", ""),
    ];

    for (msg, _status, color) in login_messages {
        if msg.is_empty() {
            term.add_line("", None).await;
        } else if msg.contains("login:") {
            term.add_line(msg, Some(LineOptions::new().with_typing(50)))
                .await;
        } else if msg.contains("password:") {
            let full = format!("{}••••••••", msg);
            term.add_line(&full, Some(LineOptions::new().with_typing(50)))
                .await;
        } else {
            term.add_line(msg, Some(LineOptions::new().with_color(color)))
                .await;
        }
        term.sleep(60).await;
    }
}
