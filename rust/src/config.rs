//! Configuration management for the GitHub User Fetcher CLI tool.
//!
//! This module provides a [`Config`] struct for managing runtime
//! configuration parsed from command-line arguments using `clap`.
//!
//! Depending on the provided arguments, the application runs in either
//! server mode or CLI mode.
//!
//! # Examples
//!
//! ```bash
//! cargo run -- --user octocat
//! cargo run -- --server --port 8080
//! ```
use clap::Parser;
use std::ffi::OsString;

const DEFAULT_USERNAME: &str = "izelnakri";
const DEFAULT_PORT: u16 = 1234;

/// Runtime configuration determined from command-line arguments.
///
/// The application runs in either:
/// - [`Mode::Server`] for running an HTTP server
/// - [`Mode::Cli`] to fetch GitHub user info directly
#[derive(Debug, Clone)]
pub struct Config {
    /// Operation mode: CLI or Server
    pub mode: Mode,
    /// Port used in server mode
    pub port: u16,
    /// GitHub username to fetch in CLI mode
    pub username: String,
}

/// Possible runtime modes of the application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Server,
    Cli,
}

#[derive(Debug, Clone, Parser)]
#[command(
    name = "github_user_fetcher",
    about = "Fetch GitHub user info or run an HTTP server",
    ignore_errors(true)
)]
struct Args {
    /// Run as HTTP server
    #[arg(long)]
    server: bool,

    /// Port for HTTP server
    #[arg(long, default_value_t = DEFAULT_PORT)]
    port: u16,

    /// GitHub username to fetch
    #[arg(long, default_value = DEFAULT_USERNAME)]
    user: String,
}

impl Config {
    /// Parse configuration from command-line arguments.
    ///
    /// Accepts any iterator over items convertible into `OsString`, including
    /// `std::env::args()`, slices or vectors of `&str` or `String`.
    ///
    /// # Examples
    ///
    /// ```
    /// use github_user_fetcher::config::{Config, Mode};
    ///
    /// // Using std::env::args()
    /// let config = Config::from_args(std::env::args());
    /// assert!(matches!(config.mode, Mode::Cli)); // Default mode
    ///
    /// // Using a vector of &str, no .to_string() needed
    /// let args = vec!["test_bin", "--server", "--port", "8080"];
    /// let config = Config::from_args(args);
    /// assert!(matches!(config.mode, Mode::Server));
    /// assert_eq!(config.port, 8080);
    /// ```
    pub fn from_args<I, T>(args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let parsed = Args::parse_from(args);

        if parsed.server {
            Config {
                mode: Mode::Server,
                port: parsed.port,
                username: String::new(),
            }
        } else {
            Config {
                mode: Mode::Cli,
                port: 0,
                username: parsed.user,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_args(args: &[&str]) -> Config { // Insert a dummy argv[0] at front (program name)
        let mut full_args = vec!["test_bin"];
        full_args.extend_from_slice(args);
        Config::from_args(full_args)
    }

    #[test]
    fn test_cli_mode_with_default_user() {
        let config = parse_args(&[]);

        assert!(matches!(config.mode, Mode::Cli));
        assert_eq!(config.username, "izelnakri");
        assert_eq!(config.port, 0);
    }

    #[test]
    fn test_cli_mode_with_custom_user() {
        let config = parse_args(&["--user", "octocat"]);

        assert!(matches!(config.mode, Mode::Cli));
        assert_eq!(config.username, "octocat");
        assert_eq!(config.port, 0);
    }

    #[test]
    fn test_server_mode_with_default_port() {
        let config = parse_args(&["--server"]);

        assert!(matches!(config.mode, Mode::Server));
        assert_eq!(config.port, 1234);
        assert!(config.username.is_empty());
    }

    #[test]
    fn test_server_mode_with_custom_port() {
        let config = parse_args(&["--server", "--port", "8080"]);

        assert!(matches!(config.mode, Mode::Server));
        assert_eq!(config.port, 8080);
        assert!(config.username.is_empty());
    }

    #[test]
    fn test_cli_ignores_port_flag() {
        let config = parse_args(&["--port", "9999", "--user", "ghuser"]);

        assert!(matches!(config.mode, Mode::Cli));
        assert_eq!(config.username, "ghuser");
        assert_eq!(config.port, 0);
    }

    #[test]
    fn test_direct_vec_args_call() {
        // directly pass Vec<&str>
        let args = vec!["test_bin", "--server", "--port", "4567"];
        let config = Config::from_args(args);

        assert!(matches!(config.mode, Mode::Server));
        assert_eq!(config.port, 4567);
        assert!(config.username.is_empty());
    }

    #[test]
    fn test_direct_env_args_call() {
        let args = std::env::args();
        let config = Config::from_args(args);
        // We can't assert exact mode here as it depends on runtime args,
        // but this just confirms it compiles and runs.
        drop(config);
    }
}
