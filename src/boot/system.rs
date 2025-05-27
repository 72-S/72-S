use crate::terminal::Terminal;

pub async fn run_system_boot(term: &Terminal) {
    let boot_messages = vec![
        ("Loading Linux linux.", "[OK]", "green"),
        ("Loading initial ramdisk.", "[OK]", "green"),
        (
            "Starting systemd-udevd version 254.5-1-arch",
            "[OK]",
            "green",
        ),
        ("Found device /dev/sda1", "[OK]", "green"),
        ("Started systemd-fsck@dev-disk-by\\x2duuid", "[OK]", "green"),
        ("Mounting /boot.", "[OK]", "green"),
        ("Mounting /home.", "[OK]", "green"),
        ("Starting Create System Users.", "[OK]", "green"),
        (
            "Starting systemd-tmpfiles-setup-dev.service",
            "[OK]",
            "green",
        ),
        ("Starting Load Kernel Modules.", "[OK]", "green"),
        ("Started systemd-journald.service", "[OK]", "green"),
        (
            "Starting Flush Journal to Persistent Storage.",
            "[OK]",
            "green",
        ),
        ("Starting Network Time Synchronization.", "[OK]", "green"),
        ("Starting User Login Management.", "[OK]", "green"),
    ];

    for (msg, status, color) in boot_messages {
        term.add_line_boot(msg, status, color).await;
        term.sleep(50).await;
    }

    term.add_line("").await;
    term.add_line_boot("Starting objz Portfolio System", "[OK]", "green")
        .await;
    term.sleep(200).await;
    term.add_line("").await;
}
