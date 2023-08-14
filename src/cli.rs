use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use crate::checkers::base::Action;

use super::checkers::read_checks_from_path;

#[derive(Copy, Clone)]
pub enum ExitStatus {
    /// Linting was successful and there were no linting errors.
    Success,
    /// Linting was successful but there were linting errors.
    Failure,
    /// Linting failed.
    Error,
}

impl From<ExitStatus> for ExitCode {
    fn from(status: ExitStatus) -> Self {
        match status {
            ExitStatus::Success => ExitCode::from(0),
            ExitStatus::Failure => ExitCode::from(1),
            ExitStatus::Error => ExitCode::from(2),
        }
    }
}

/// Config Checker will check and optional fix your config files based on checkers defined in a toml file.
/// It can check toml, yaml, json and plain text files.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the root checkers file in toml format
    #[arg(short, long, default_value = "checkers.toml")]
    path: String,

    /// Try to fix the config
    #[arg(long, default_value = "false")]
    fix: bool,
}

pub fn cli() -> ExitCode {
    simple_logger::init().unwrap();
    log::info!("Starting check-config");

    let cli = Cli::parse();
    let file_with_checks = &PathBuf::from(&cli.path);

    log::info!(
        "Using checkers from {}",
        &file_with_checks.to_string_lossy()
    );
    log::info!("Fix: {}", &cli.fix);

    let checks = read_checks_from_path(file_with_checks);

    let mut action_count = 0;

    for check in checks {
        if cli.fix {
            match check.fix() {
                Err(_) => return ExitCode::from(2),
                Ok(Action::None) => {}
                _ => action_count += 1,
            };
        } else {
            match check.check() {
                Err(_) => return ExitCode::from(2),
                Ok(Action::None) => {}
                _ => action_count += 1,
            }
        };
    }
    if action_count > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::from(0)
    }
}
