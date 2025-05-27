use crate::terminal::Terminal;

pub async fn run_boot_init(term: &Terminal) {
    let init_messages = vec![
        ("Starting portfolio services", "[OK]", "green"),
        ("Loading project database", "[OK]", "green"),
        ("Initializing web terminal", "[OK]", "green"),
        ("Setting up command processor", "[OK]", "green"),
        ("Establishing connections", "[OK]", "green"),
    ];

    for (msg, status, color) in init_messages {
        term.add_line_boot(msg, status, color).await;
        term.sleep(60).await;
    }
}
