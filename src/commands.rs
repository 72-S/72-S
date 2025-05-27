use crate::ascii_art::AsciiArt;

pub struct CommandProcessor {
    history: Vec<String>,
    ascii_art: AsciiArt,
}

impl Clone for CommandProcessor {
    fn clone(&self) -> Self {
        Self {
            history: self.history.clone(),
            ascii_art: self.ascii_art.clone(),
        }
    }
}

impl CommandProcessor {
    pub fn new() -> Self {
        let ascii_art = AsciiArt::new();

        Self {
            history: Vec::new(),
            ascii_art,
        }
    }

    pub fn process_command(&mut self, input: &str) -> String {
        let input = input.trim();
        if input.is_empty() {
            return String::new();
        }

        self.history.push(input.to_string());

        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts[0];
        let args = &parts[1..];

        match command {
            "help" => Self::help(args),
            "whoami" => Self::whoami(args),
            "clear" => "CLEAR_SCREEN".to_string(),
            "history" => self.history_cmd(args),
            "uptime" => Self::uptime(args),
            "top" => Self::top(args),
            "ls" => Self::ls(args),
            "cat" => Self::cat(args),
            "ascii" => self.ascii_art.get_ascii(args),
            "sudo" => Self::sudo(args),
            "make" => Self::make(args),
            "telnet" => Self::telnet(args),
            "nc" | "netcat" => Self::netcat(args),
            "echo" => Self::echo(args),
            "pwd" => "/home/objz".to_string(),
            "date" => "Mon May 27 13:28:47 UTC 2025".to_string(),
            "ps" => Self::ps(args),
            "neofetch" => Self::neofetch(args),
            "matrix" => self.ascii_art.get_matrix_effect(),
            "hack" => Self::hack(args),
            _ => format!("bash: {}: command not found", command),
        }
    }

    fn help(_args: &[&str]) -> String {
        r#"Available commands:

NAVIGATION:
  ls [path]           - List directory contents
  cat <file>         - Display file contents
  pwd                - Show current directory
  
SYSTEM:
  whoami             - Display current user
  uptime             - Show system uptime
  top                - Display running processes
  ps                 - Show process list
  history            - Show command history
  clear              - Clear the terminal
  neofetch           - System information
  date               - Show current date
  
PORTFOLIO:
  ascii <topic>      - Display ASCII art
  matrix             - Enter the matrix
  
NETWORK:
  telnet <host>      - Connect to host
  nc <host> <port>   - Netcat connection
  
EASTER EGGS:
  sudo rm -rf /      - Don't try this at home!
  make coffee        - Brew some coffee
  hack               - Initiate hacking sequence
  echo <text>        - Display text (try echo $USER)
  
Type any command to get started!"#
            .to_string()
    }

    fn whoami(_args: &[&str]) -> String {
        r#"objz (Object-Oriented Developer)
Software Engineer & System Architect
Currently based in Germany

Specializing in: Rust, Java, TypeScript, System Design
Working on: CommandBridge, MCL, Portfolio projects

"Code is like humor. When you have to explain it, it's bad." - Cory House"#
            .to_string()
    }

    fn history_cmd(&self, _args: &[&str]) -> String {
        if self.history.is_empty() {
            "No commands in history yet.".to_string()
        } else {
            self.history
                .iter()
                .enumerate()
                .map(|(i, cmd)| format!("  {}  {}", i + 1, cmd))
                .collect::<Vec<_>>()
                .join("\n")
        }
    }

    fn uptime(_args: &[&str]) -> String {
        "Portfolio uptime: 17 days, 13:28, load average: 0.42, 0.13, 0.37".to_string()
    }

    fn top(_args: &[&str]) -> String {
        r#"  PID USER      %CPU  %MEM  COMMAND
 1337 objz      12.3   4.2  ./portfolio_server
 1338 objz       8.7   2.1  ./command_processor  
 1339 objz       5.4   1.8  ./ascii_engine
 1340 objz       3.2   0.9  ./animation_handler
 1341 objz       0.5   0.3  ./project_showcase
 1342 objz       0.1   0.1  ./coffee_maker"#
            .to_string()
    }

    fn ps(_args: &[&str]) -> String {
        r#"  PID TTY          TIME CMD
 1337 pts/0    00:00:42 portfolio
 1338 pts/0    00:00:21 wasm-runtime
 1339 pts/0    00:00:18 terminal-emu
 1340 pts/0    00:00:12 animation-sys
 1341 pts/0    00:00:05 ascii-render"#
            .to_string()
    }

    fn neofetch(_args: &[&str]) -> String {
        r#"                   -`                    objz@portfolio
                  .o+`                   -----------------
                 `ooo/                   OS: Portfolio Linux x86_64
                `+oooo:                  Host: GitHub Pages
               `+oooooo:                 Kernel: WASM 6.6.6-portfolio
               -+oooooo+:                Uptime: 17 days, 13 hours, 28 mins
             `/:-:++oooo+:               Packages: 42 (rust), 13 (npm)
            `/++++/+++++++:              Shell: portfolio-shell 3.0.0
           `/++++++++++++++:             Resolution: 1920x1080
          `/+++ooooooooo++++/            WM: Terminal Emulator
         ./ooosssso++osssssso+`          Theme: Matrix-Dark
        .oossssso-````/ossssss+`         Icons: ASCII Art Pack
       -osssssso.      :ssssssso.        Terminal: portfolio-term
      :osssssss/        osssso+++.       CPU: WASM Virtual Core
     /ossssssss/        +ssssooo/-       Memory: 521MiB / ∞GiB
   `/ossssso+/:-        -:/+osssso+-     
  `+sso+:-`                 `.-/+oso:    
 `++:.                           `-/+/   
 .`                                 `/   "#
            .to_string()
    }

    fn ls(args: &[&str]) -> String {
        let path = args.get(0).unwrap_or(&"~");

        match *path {
            "~" | "." => "projects/  skills/  contact/  README.md".to_string(),
            "~/projects" | "projects" => {
                "commandbridge.md  mcl.md  notenmanager.md  dots.md  README.md".to_string()
            }
            "~/skills" | "skills" => "languages.txt  frameworks.txt  tools.txt".to_string(),
            _ => format!("ls: cannot access '{}': No such file or directory", path),
        }
    }

    fn cat(args: &[&str]) -> String {
        if args.is_empty() {
            return "cat: missing file operand".to_string();
        }

        let file = args[0];
        match file {
            "README.md" | "~/README.md" => r#"# objz's Portfolio Terminal

Welcome to my interactive portfolio! This terminal simulates a Linux environment
where you can explore my projects and skills.

## Quick Navigation:
- `ls ~/projects` - View my projects
- `cat ~/projects/<project>.md` - Read project details  
- `whoami` - Learn about me
- `help` - See all commands

Enjoy exploring!"#
                .to_string(),

            "~/projects/commandbridge.md" | "projects/commandbridge.md" | "commandbridge.md" => {
                format!(
                    "{}\n\n{}",
                    AsciiArt::get_project_ascii("bridge"),
                    r#"# CommandBridge
A powerful bridge system between Paper and Velocity Minecraft servers.

## Features:
- Cross-server command execution
- Permission synchronization  
- Real-time player communication
- Plugin API for extensions

## Tech Stack: Java, Paper API, Velocity API
## Status: Active Development
## GitHub: https://github.com/objz/commandbridge

"The bridge between worlds of Minecraft servers.""#
                )
            }

            "~/projects/mcl.md" | "projects/mcl.md" | "mcl.md" => {
                format!(
                    "{}\n\n{}",
                    AsciiArt::get_project_ascii("rust"),
                    r#"# mcl - Minecraft CLI Launcher
A fast, efficient Minecraft launcher written in Rust.

## Features:
- Lightning-fast startup
- Multiple profile management
- Mod support
- Cross-platform compatibility

## Tech Stack: Rust, Tokio, CLI
## Status: Beta
## GitHub: https://github.com/objz/mcl

"Because launching Minecraft should be as fast as Rust.""#
                )
            }

            "~/projects/notenmanager.md" | "projects/notenmanager.md" | "notenmanager.md" => {
                format!(
                    "{}\n\n{}",
                    AsciiArt::get_project_ascii("firebase"),
                    r#"# Notenmanager
Firebase-powered school grade tracking application.

## Features:
- Real-time grade synchronization
- Statistical analysis
- Multi-user support
- Offline capabilities

## Tech Stack: Firebase, JavaScript, PWA
## Status: Production
## Use Case: Academic grade management

"Keeping track of academic success, one grade at a time.""#
                )
            }

            "~/projects/dots.md" | "projects/dots.md" | "dots.md" => {
                format!(
                    "{}\n\n{}",
                    AsciiArt::get_project_ascii("linux"),
                    r#"# dots - Hyprland Dotfiles
Automated dotfiles setup for Hyprland window manager.

## Features:
- One-command installation
- Hyprland configuration
- Custom theming
- Backup system

## Tech Stack: Shell, Hyprland, Linux
## Status: Maintained
## GitHub: https://github.com/objz/dots

"Making Linux beautiful, one config at a time.""#
                )
            }

            _ => format!("cat: {}: No such file or directory", file),
        }
    }

    fn sudo(args: &[&str]) -> String {
        if args.len() >= 3 && args[0] == "rm" && args[1] == "-rf" && args[2] == "/" {
            "SYSTEM_PANIC".to_string()
        } else {
            "[sudo] password for objz: \n\nSorry, try again.\n[sudo] password for objz: \n\nSudo access denied for portfolio demo.".to_string()
        }
    }

    fn make(args: &[&str]) -> String {
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

    fn echo(args: &[&str]) -> String {
        if args.is_empty() {
            String::new()
        } else if args[0] == "$USER" {
            AsciiArt::get_user_ascii()
        } else {
            args.join(" ")
        }
    }

    fn telnet(args: &[&str]) -> String {
        let host = args.get(0).unwrap_or(&"localhost");
        if *host == "127.0.0.1" || *host == "localhost" {
            r#"Trying 127.0.0.1...
Connected to localhost.
Escape character is '^]'.

░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
░  SECURE TERMINAL ACCESS GRANTED  ░
░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░

You are now connected to the portfolio mainframe.
Type 'exit' to disconnect.

mainframe> Welcome to the objz portfolio system
mainframe> All activities are monitored
mainframe> Have a nice day :)

Connection closed by foreign host."#
                .to_string()
        } else {
            format!(
                "telnet: could not resolve {}/telnet: Name or service not known",
                host
            )
        }
    }

    fn netcat(args: &[&str]) -> String {
        if args.len() >= 2 {
            let host = args[0];
            let port = args[1];

            if host.contains("hacker") && port == "1337" {
                r#"Connection established to hacker.net:1337

    ██╗  ██╗ █████╗  ██████╗██╗  ██╗███████╗██████╗ 
    ██║  ██║██╔══██╗██╔════╝██║ ██╔╝██╔════╝██╔══██╗
    ███████║███████║██║     █████╔╝ █████╗  ██████╔╝
    ██╔══██║██╔══██║██║     ██╔═██╗ ██╔══╝  ██╔══██╗
    ██║  ██║██║  ██║╚██████╗██║  ██╗███████╗██║  ██║
    ╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝
                                                    
    Welcome to the hacker matrix...
    Access level: GUEST
    Available exploits: 0
    
    > This is just a portfolio demo!
    > No actual hacking here :)
    
Connection terminated."#
                    .to_string()
            } else {
                format!("nc: connect to {} port {}: Connection refused", host, port)
            }
        } else {
            "usage: nc [-options] hostname port".to_string()
        }
    }

    fn hack(_args: &[&str]) -> String {
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
}
