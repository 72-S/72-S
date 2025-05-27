pub fn help(_args: &[&str]) -> String {
    r#"Available commands:

NAVIGATION:
  ls [path]           - List directory contents
  cat <file>          - Display file contents
  pwd                 - Show current directory
  
SYSTEM:
  whoami              - Display current user
  uptime              - Show system uptime
  top                 - Display running processes
  ps                  - Show process list
  history             - Show command history
  clear               - Clear the terminal
  neofetch            - System information
  date                - Show current date
  
PORTFOLIO:
  ascii <topic>       - Display ASCII art
  matrix              - Enter the matrix
  
NETWORK:
  telnet <host>       - Connect to host
  nc <host> <port>    - Netcat connection
  
EASTER EGGS:
  sudo rm -rf /       - Don't try this at home!
  make coffee         - Brew some coffee
  hack                - Initiate hacking sequence
  echo <text>         - Display text (try echo $USER)
  
Type any command to get started!"#
        .to_string()
}

pub fn whoami(_args: &[&str]) -> String {
    r#"objz (Object-Oriented Developer)
Software Engineer & System Architect
Currently based in Germany

Specializing in: Rust, Java, TypeScript, System Design
Working on: CommandBridge, MCL, Portfolio projects

"Code is like humor. When you have to explain it, it's bad." - Cory House"#
        .to_string()
}

pub fn clear(_args: &[&str]) -> String {
    "CLEAR_SCREEN".to_string()
}

pub fn date(_args: &[&str]) -> String {
    "Mon May 27 13:28:47 UTC 2025".to_string()
}

pub fn uptime(_args: &[&str]) -> String {
    "Portfolio uptime: 17 days, 13:28, load average: 0.42, 0.13, 0.37".to_string()
}

pub fn top(_args: &[&str]) -> String {
    r#"  PID USER      %CPU  %MEM  COMMAND
 1337 objz      12.3   4.2  ./portfolio_server
 1338 objz       8.7   2.1  ./command_processor  
 1339 objz       5.4   1.8  ./ascii_engine
 1340 objz       3.2   0.9  ./animation_handler
 1341 objz       0.5   0.3  ./project_showcase
 1342 objz       0.1   0.1  ./coffee_maker"#
        .to_string()
}

pub fn ps(_args: &[&str]) -> String {
    r#"  PID TTY          TIME CMD
 1337 pts/0    00:00:42 portfolio
 1338 pts/0    00:00:21 wasm-runtime
 1339 pts/0    00:00:18 terminal-emu
 1340 pts/0    00:00:12 animation-sys
 1341 pts/0    00:00:05 ascii-render"#
        .to_string()
}

pub fn neofetch(_args: &[&str]) -> String {
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
      :osssssss/        +sssso+++.
     /ossssssss/        +ssssooo/-       Memory: 521MiB / ∞GiB
   `/ossssso+/:-        -:/+osssso+-     
  `+sso+:-`                 `.-/+oso:    
 `++:.                           `-/+/   
 .`                                 `/   "#
        .to_string()
}
