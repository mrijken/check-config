use std::io::Write;
use std::process::ExitCode;
use std::sync::Arc;

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
    #[arg(short, long, default_value = "check_config.toml")]
    path: String,

    /// Try to fix the config
    #[arg(long, default_value = "false")]
    fix: bool,

    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

pub fn cli() -> ExitCode {
    let cli = Cli::parse();
    let cwd = super::uri::parse_uri(
        &format!(
            "file://{}/",
            std::env::current_dir().unwrap().as_path().to_str().unwrap()
        ),
        None,
    )
    .unwrap();
    let mut file_with_checks = match super::uri::parse_uri(&cli.path, Some(&cwd)) {
        Ok(uri) => uri,
        Err(_) => {
            log::error!("Unable to load checkers. Invalid path: {}", cli.path);
            return ExitCode::from(ExitStatus::Error);
        }
    };

    if !std::path::Path::new(file_with_checks.path()).exists() {
        if !std::path::Path::new("pyproject.toml").exists() {
            log::error!(
                "Path with checkers does not exist: {}",
                file_with_checks.path()
            );
            return ExitCode::from(ExitStatus::Error);
        }
        log::info!(
            "Path with checkers does not exist: {}",
            file_with_checks.path()
        );
        log::info!("Using pyproject.toml as alternative");
        file_with_checks = super::uri::parse_uri("pyproject.toml", Some(&cwd)).unwrap();
    }

    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .init();
    log::info!("Starting check-config");

    log::info!("Using checkers from {}", &file_with_checks);
    log::info!("Fix: {}", &cli.fix);

    let checks = read_checks_from_path(&file_with_checks);

    let mut action_count = 0;
    let mut success_count = 0;

    for check in checks {
        let result = if cli.fix { check.fix() } else { check.check() };

        match result {
            Err(_) => {
                log::error!("There was an error fixing files.");
                return ExitCode::from(ExitStatus::Error);
            }
            Ok(Action::None) => success_count += 1,
            _ => action_count += 1,
        };
    }
    log::info!("{} checks successful.", success_count);
    if action_count > 0 {
        // note: error level is used to always show this message, also with the lowest verbose level
        log::error!("There are {} violations to fix.", action_count,);
        ExitCode::from(ExitStatus::Failure)
    } else {
        // note: error level is used to always show this message, also with the lowest verbose level
        log::error!("No violations found.");
        ExitCode::from(ExitStatus::Success)
    }
}
