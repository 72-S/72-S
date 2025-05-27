pub fn sudo(args: &[&str]) -> String {
    if args.len() >= 3 && args[0] == "rm" && args[1] == "-rf" && args[2] == "/" {
        "SYSTEM_PANIC".to_string()
    } else {
        "[sudo] password for objz: \n\nSorry, try again.\n[sudo] password for objz: \n\nSudo access denied for portfolio demo.".to_string()
    }
}

pub fn help(_args: &[&str]) -> String {
    r#"Available commands:


Type any command to get started!"#
        .to_string()
}
