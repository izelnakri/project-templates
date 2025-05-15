use clap::{Arg, ArgAction, Command};

const DEFAULT_USERNAME: &str = "izelnakri";
const DEFAULT_PORT: u16 = 1234;

#[derive(Debug, Clone)]
pub struct Config {
    pub mode: Mode,
    pub port: u16,
    pub username: String,
}

#[derive(Debug, Clone)]
pub enum Mode {
    Server,
    Cli,
}

impl Config {
    pub fn from_args() -> Self {
        let matches = Command::new("ghfetch-rs")
            .about("Fetch GitHub user info or run an HTTP server")
            .arg(Arg::new("server")
                .long("server")
                .help("Run as HTTP server")
                .action(ArgAction::SetTrue))
            .arg(Arg::new("port")
                .long("port")
                .value_parser(clap::value_parser!(u16))
                .help("Port for HTTP server (default: 1234)"))
            .arg(Arg::new("user")
                .index(1)
                .help("GitHub username to fetch (default: izelnakri)"))
            .get_matches();

        if matches.get_flag("server") {
            let port = *matches.get_one::<u16>("port").unwrap_or(&DEFAULT_PORT);
            Self {
                mode: Mode::Server,
                port,
                username: String::new(), // not used
            }
        } else {
            let username = matches
                .get_one::<String>("user")
                .cloned()
                .unwrap_or_else(|| DEFAULT_USERNAME.to_string());
            Self {
                mode: Mode::Cli,
                port: 0, // not used
                username,
            }
        }
    }
}
