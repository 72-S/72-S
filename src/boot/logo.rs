use crate::terminal::Terminal;

pub async fn show_logo(term: &Terminal) {
    let logo_lines = vec![
        "  ___  ___ ___  ____",
        " / _ \\| _ ) _ \\|_  /",
        "| (_) | _ \\   / / / ",
        " \\___/|___/_|_\\/___|",
        "",
    ];

    for line in logo_lines {
        if line.starts_with(' ')
            && (line.contains('/') || line.contains('\\') || line.contains('|'))
        {
            term.add_colored_line_typing(line, 30, "cyan").await;
        } else {
            term.add_line(line).await;
        }
        term.sleep(40).await;
    }
}
