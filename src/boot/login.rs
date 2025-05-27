use crate::terminal::Terminal;

pub async fn run_login_sequence(term: &Terminal) {
    let login_messages = vec![
        ("Arch Linux 6.6.32-1-lts (tty1)", "", "green"),
        ("", "", ""),
        ("objz-portfolio login: objz", "", "white"),
        ("Password: ", "", "white"),
        ("", "", ""),
        ("Last login: Mon May 27 13:59:36 2024", "", "white"),
        ("", "", ""),
    ];

    for (msg, _status, color) in login_messages {
        if msg.is_empty() {
            term.add_line("").await;
        } else if msg.contains("login:") {
            term.add_line_typing(msg, 50).await;
        } else if msg.contains("Password:") {
            let full = format!("{}••••••••", msg);
            term.add_line_typing(&full, 50).await;
        } else {
            term.add_line_colored(msg, color).await;
        }
        term.sleep(60).await;
    }
}
