use crate::ascii_art::AsciiArt;

pub fn list(args: &[&str]) -> String {
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

pub fn display(args: &[&str]) -> String {
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
                AsciiArt::new().get_ascii(&["bridge"]),
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
                AsciiArt::new().get_ascii(&["rust"]),
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
                AsciiArt::new().get_ascii(&["firebase"]),
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
                AsciiArt::new().get_ascii(&["linux"]),
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
