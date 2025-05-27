pub fn telnet(args: &[&str]) -> String {
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

pub fn netcat(args: &[&str]) -> String {
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
