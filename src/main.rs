mod config;
mod event_tap;
mod service;

use std::process::ExitCode;

use clap::{Parser, Subcommand};

use crate::config::Config;

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run ScrollSplit in the foreground.
    Run,
    /// Start the installed LaunchAgent.
    Start,
    /// Stop the installed LaunchAgent.
    Stop,
    /// Restart the installed LaunchAgent.
    Restart,
    /// Show LaunchAgent status.
    Status,
    /// Read or update configuration.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// Install and start the user LaunchAgent.
    InstallService,
    /// Stop and remove the user LaunchAgent.
    UninstallService,
}

#[derive(Subcommand)]
enum ConfigCommand {
    /// Print the current configuration.
    Show,
    /// Set a boolean configuration value.
    Set { key: String, value: String },
}

fn main() -> ExitCode {
    match execute(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("scrollsplit: {error}");
            ExitCode::FAILURE
        }
    }
}

fn execute(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Run => event_tap::run(),
        Command::Start => service::start(),
        Command::Stop => service::stop(),
        Command::Restart => service::restart(),
        Command::Status => service::status(),
        Command::Config { command } => match command {
            ConfigCommand::Show => {
                let config = Config::load_or_create()?;
                print!("{}", config.to_toml()?);
                Ok(())
            }
            ConfigCommand::Set { key, value } => {
                let value = parse_bool(&value)?;
                let mut config = Config::load_or_create()?;
                config.set(&key, value)?;
                config.save()?;
                println!("{key} = {value}");
                Ok(())
            }
        },
        Command::InstallService => service::install(),
        Command::UninstallService => service::uninstall(),
    }
}

fn parse_bool(value: &str) -> Result<bool, String> {
    match value.to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => Err(format!("expected a boolean, got {value:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_bool;

    #[test]
    fn parses_common_boolean_values() {
        assert_eq!(parse_bool("true"), Ok(true));
        assert_eq!(parse_bool("OFF"), Ok(false));
        assert!(parse_bool("maybe").is_err());
    }
}
