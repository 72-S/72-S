#[derive(Clone)]
pub struct AsciiArt;

impl AsciiArt {
    pub fn new() -> Self {
        Self
    }

    pub fn get_ascii(&self, args: &[&str]) -> String {
        if args.is_empty() {
            return "Usage: ascii <topic>\nAvailable topics: rust, linux, hacker, coffee, matrix, bridge, firebase".to_string();
        }

        let topic = args[0];
        Self::get_project_ascii(topic)
    }

    pub fn get_matrix_effect(&self) -> String {
        r#"
    ░░▒▒▓▓██ ENTERING THE MATRIX ██▓▓▒▒░░
    
    01001000 01100101 01101100 01101100 01101111
    
    ░█▀▀█ ░█▀▀█ ░█▀▀█ ▀▀█▀▀ ░█▀▀▀ ░█▀▀█ ░█─── ▀█▀ ░█▀▀█ 
    ░█▄▄█ ░█▄▄█ ░█▄▄▀ ─░█── ░█▀▀▀ ░█▄▄█ ░█─── ░█─ ░█▄▄█ 
    ░█─── ░█─░█ ░█─░█ ─░█── ░█─── ░█─░█ ░█▄▄█ ▄█▄ ░█─░█
    
    01010111 01101111 01110010 01101100 01100100
    
    > Wake up, developer... The portfolio has you.
    > There is no spoon... only code.
    > Follow the white rabbit (🐰) to ~/projects
    
    Matrix connection established.
    Red pill or blue pill? Type 'help' to choose.
        "#
        .to_string()
    }

    pub fn get_user_ascii() -> String {
        r#"
  ___  ___ ___  ____
 / _ \| _ ) _ \|_  /
| (_) | _ \   / / / 
 \___/|___/_|_\/___|
                    
    [ objz@portfolio ]
    
    "Code is poetry in motion"
        "#
        .to_string()
    }

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
"#.to_string(),

            "rust" => r#"
    ⚡ RUST POWERED ⚡
    
    ██████╗ ██╗   ██╗███████╗████████╗
    ██╔══██╗██║   ██║██╔════╝╚══██╔══╝
    ██████╔╝██║   ██║███████╗   ██║   
    ██╔══██╗██║   ██║╚════██║   ██║   
    ██║  ██║╚██████╔╝███████║   ██║   
    ╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝   
    
    🦀 Fast • Safe • Concurrent 🦀
"#.to_string(),

            "firebase" => r#"
         🔥 Firebase Powered 🔥
    
      ░░░░░░░░░░░░░░░░░░░░░░░░░
      ░  REALTIME DATABASE  ░
      ░   ┌─────────────┐   ░
      ░   │ Notenmanager│   ░
      ░   │     📊      │   ░
      ░   │   Grades    │   ░
      ░   └─────────────┘   ░
      ░░░░░░░░░░░░░░░░░░░░░░░░░
"#.to_string(),

            "linux" => r#"
        🐧 LINUX CONFIGURATION 🐧
    
         ██╗     ██╗███╗   ██╗██╗   ██╗██╗  ██╗
         ██║     ██║████╗  ██║██║   ██║╚██╗██╔╝
         ██║     ██║██╔██╗ ██║██║   ██║ ╚███╔╝ 
         ██║     ██║██║╚██╗██║██║   ██║ ██╔██╗ 
         ███████╗██║██║ ╚████║╚██████╔╝██╔╝ ██╗
         ╚══════╝╚═╝╚═╝  ╚═══╝ ╚═════╝ ╚═╝  ╚═╝
    
         Hyprland • Dotfiles • Automation
"#.to_string(),

            "hacker" => r#"
    ░█▀▀█ ░█─░█ ░█▀▀▄ ░█▀▀▀ ░█▀▀█ 　 ░█─░█ ░█▀▀█ ░█▀▀█ ░█─▄▀ ░█▀▀▀ ░█▀▀█ 
    ░█─── ░█▄▄█ ░█▀▀▄ ░█▀▀▀ ░█▄▄▀ 　 ░█▀▀█ ░█▄▄█ ░█─── ░█▀▄─ ░█▀▀▀ ░█▄▄▀ 
    ░█▄▄█ ─▀─▀─ ░█▄▄▀ ░█▄▄▄ ░█─░█ 　 ░█─░█ ░█─░█ ░█▄▄█ ░█─░█ ░█▄▄▄ ░█─░█
    
    [ACCESSING MAINFRAME...] ████████████ 100%
    [BYPASSING FIREWALL...]  ██████████░░  83%
    [DECRYPTING DATA...]     ████████░░░░  67%
    
    > Just kidding! This is just ASCII art :)
"#.to_string(),

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
"#.to_string(),

            "matrix" => r#"
    ░░▒▒▓▓██ THE MATRIX ██▓▓▒▒░░
    
    01001000 01100101 01101100 01101100 01101111
    ░█▀▀█ ░█▀▀█ ░█▀▀█ ▀▀█▀▀ ░█▀▀▀ ░█▀▀█ ░█─── ▀█▀ ░█▀▀█ 
    ░█▄▄█ ░█▄▄█ ░█▄▄▀ ─░█── ░█▀▀▀ ░█▄▄█ ░█─── ░█─ ░█▄▄█ 
    ░█─── ░█─░█ ░█─░█ ─░█── ░█─── ░█─░█ ░█▄▄█ ▄█▄ ░█─░█
    01010111 01101111 01110010 01101100 01100100
    
    > Wake up, developer... The portfolio has you.
"#.to_string(),

            _ => format!("ASCII art for '{}' not found. Try: rust, linux, hacker, coffee, matrix, bridge, firebase", topic),
        }
    }
}
