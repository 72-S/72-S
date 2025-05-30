use crate::terminal::{renderer::LineOptions, Terminal};

pub async fn system_panic(terminal: &Terminal) {
    terminal.clear_output();

    let panic_lines = vec![
        ("⚠️  CRITICAL SYSTEM ERROR ⚠️", Some("error"), None),
        ("", None, None),
        ("Deleting root filesystem...", Some("warning"), Some(100)),
        (
            "rm: removing /usr... ████████████░░░░ 75%",
            Some("warning"),
            Some(80),
        ),
        (
            "rm: removing /var... ██████████████░░ 87%",
            Some("warning"),
            Some(80),
        ),
        (
            "rm: removing /etc... ████████████████ 100%",
            Some("warning"),
            Some(80),
        ),
        ("", None, None),
        ("SYSTEM DESTROYED ☠️", Some("error"), Some(150)),
        ("", None, None),
        (
            "Just kidding! This is a just website, not your actual system.",
            Some("success"),
            Some(50),
        ),
        ("Nice try though! 😉", Some("success"), None),
        ("", None, None),
        (
            "(Don't actually run 'sudo rm -rf /' on real systems!)",
            Some("warning"),
            None,
        ),
        ("", None, None),
    ];

    for (line, color, typing_speed) in panic_lines {
        let mut options = LineOptions::new();

        if let Some(color_class) = color {
            options = options.with_color(color_class);
        }

        if let Some(speed) = typing_speed {
            options = options.with_typing(speed);
        }

        terminal.add_line(line, Some(options)).await;

        terminal.sleep(300).await;
    }

    terminal.sleep(2000).await;

    terminal.clear_output();

    terminal
        .add_line(
            "System restored! Terminal is back online.",
            Some(LineOptions::new().with_color("success")),
        )
        .await;

    terminal.add_line("", None).await;
}
