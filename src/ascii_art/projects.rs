pub const AVAILABLE_TOPICS: &[&str] = &["bridge", "rust", "firebase", "linux", "hacker", "coffee"];

pub fn get_project_ascii(topic: &str) -> String {
    match topic {
        "bridge" => r#"
╔══════════════════════════════════╗
║  🌉 CommandBridge Network 🌉     ║
║                                  ║
║  [Paper] ←→ [Bridge] ←→ [Velocity] ║
║     ↓         ↓         ↓       ║
║  Player1   Commands   Player2    ║
╚══════════════════════════════════╝
"#
        .to_string(),

        "rust" => r#"
⚡ RUST POWERED ⚡

██████╗ ██╗   ██╗███████╗████████╗
██╔══██╗██║   ██║██╔════╝╚══██╔══╝
██████╔╝██║   ██║███████╗   ██║   
██╔══██╗██║   ██║╚════██║   ██║   
██║  ██║╚██████╔╝███████║   ██║   
╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝   

🦀 Fast • Safe • Concurrent 🦀
"#
        .to_string(),

        "firebase" => r#"
🔥 Firebase Powered 🔥

░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
░  REALTIME DATABASE  ░
░   ┌─────────────┐   ░
░   │ Notenmanager│   ░
░   │     📊      │   ░
░   │   Grades    │   ░
░   └─────────────┘   ░
░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
"#
        .to_string(),

        "linux" => r#"
🐧 LINUX CONFIGURATION 🐧

 ██╗     ██╗███╗   ██╗██╗   ██╗██╗  ██╗
 ██║     ██║████╗  ██║██║   ██║╚██╗██╔╝
 ██║     ██║██╔██╗ ██║██║   ██║ ╚███╔╝ 
 ██║     ██║██║╚██╗██║██║   ██║ ██╔██╗ 
 ███████╗██║██║ ╚████║╚██████╔╝██╔╝ ██╗
 ╚══════╝╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝  ╚═╝

 Hyprland • Dotfiles • Automation
"#
        .to_string(),

        "hacker" => r#"
░█▀▀█ ░█─░█ ░█▀▀▄ ░█▀▀▀ ░█▀▀█ 　 ░█─░█ ░█▀▀█ ░█▀▀█ ░█─▄▀ ░█▀▀▀ ░█▀▀█ 
░█─── ░█▄▄█ ░█▀▀▄ ░█▀▀▀ ░█▄▄▀ 　 ░█▀▀█ ░█▄▄█ ░█─── ░█▀▄─ ░█▀▀▀ ░█▄▄▀ 
░█▄▄█ ─▀─▀─ ░█▄▄▀ ░█▄▄▄ ░█─░█ 　 ░█─░█ ░█─░█ ░█▄▄█ ░█─░█ ░█▄▄▄ ░█─░█

[ACCESSING MAINFRAME...] ████████████ 100%
[BYPASSING FIREWALL...]  ██████████░░  83%
[DECRYPTING DATA...]     ████████░░░░  67%

> Just kidding! This is just ASCII art :)
"#
        .to_string(),

        "coffee" => r#"
☕ COFFEE.EXE LOADING ☕

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

Caffeine levels: ████████████ 100%
Productivity boost: +42%
"#
        .to_string(),

        _ => format!(
            "ASCII art for '{}' not found. Try: {}",
            topic,
            AVAILABLE_TOPICS.join(", ")
        ),
    }
}
