use crate::ascii_art::AsciiArt;

pub fn echo(args: &[&str]) -> String {
    if args.is_empty() {
        String::new()
    } else if args[0] == "$USER" {
        AsciiArt::get_user_ascii()
    } else {
        args.join(" ")
    }
}

pub fn sudo(args: &[&str]) -> String {
    if args.len() >= 3 && args[0] == "rm" && args[1] == "-rf" && args[2] == "/" {
        "SYSTEM_PANIC".to_string()
    } else {
        "[sudo] password for objz: \n\nSorry, try again.\n[sudo] password for objz: \n\nSudo access denied for portfolio demo.".to_string()
    }
}

pub fn make(args: &[&str]) -> String {
    if args.get(0) == Some(&"coffee") {
        r#"
      (  )   (   )  )
       ) (   )  (  (
       ( )  (    ) )
       _____________
      <_____________> ___
      |             |/ _ \
      |      ☕      | | |
      |               |_| |
   ___|_______________|___|___
  |_______________________|

ERROR 418: I'm a teapot

The requested entity body is short and stout.
Tip me over and pour me out.

RFC 2324 - Hyper Text Coffee Pot Control Protocol
"#
        .to_string()
    } else {
        "make: *** No targets specified and no makefile found. Stop.".to_string()
    }
}

pub fn hack(_args: &[&str]) -> String {
    r#"Initializing hacking sequence...

[████████████████████████████████] 100%

⚠️  WARNING: Hacking detected! ⚠️

Just kidding! This is a portfolio website.
Try exploring with commands like:
- ls ~/projects
- cat ~/projects/commandbridge.md
- ascii matrix
- make coffee

No actual hacking happening here! 😄"#
        .to_string()
}
