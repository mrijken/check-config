use std::io::Write;
use std::process::ExitCode;

use clap::Parser;

use crate::checkers::{
    base::{Action, Check},
    RelativeUrl,
};

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
    /// Defaults (in order of precedence)
    /// - check-config.toml
    /// - pyproject.toml with a tool.check-config key
    #[arg(short, long)]
    path: Option<String>,

    /// Try to fix the config
    #[arg(long, default_value = "false")]
    fix: bool,

    /// List all checks. Checks are not executed.
    #[arg(long, default_value = "false")]
    list_checkers: bool,

    // -v s
    // -vv show all
    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

pub(crate) fn parse_path_str_to_uri(path: &str) -> Option<url::Url> {
    if path.starts_with("/") {
        super::uri::parse_uri(format!("file://{path}").as_str(), None).ok()
    } else {
        let cwd = super::uri::parse_uri(
            &format!(
                "file://{}/",
                std::env::current_dir().unwrap().as_path().to_str().unwrap()
            ),
            None,
        )
        .unwrap();
        super::uri::parse_uri(path, Some(&cwd)).ok()
    }
}
pub fn cli() -> ExitCode {
    let cli = Cli::parse();
    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .init();

    log::info!("Starting check-config");
    let checks = match cli.path {
        Some(path_str) => match parse_path_str_to_uri(path_str.as_str()) {
            Some(uri) => match std::path::Path::new(uri.path()).exists() {
                true => {
                    log::info!("Using checkers from {}", &uri.short_url_str());
                    read_checks_from_path(&uri, vec![])
                }
                false => {
                    log::error!(
                        "⚠  Unable to load checkers. Path ({path_str}) as specified does not exist.",
                    );
                    return ExitCode::from(ExitStatus::Error);
                }
            },
            None => {
                log::error!(
                    "Unable to load checkers. Path ({path_str}) specified is not a valid path.",
                );
                return ExitCode::from(ExitStatus::Error);
            }
        },
        None => {
            log::warn!("⚠️ No path specified. Trying check-config.toml");
            let uri = parse_path_str_to_uri("check-config.toml").expect("valid path");
            match std::path::Path::new(uri.path()).exists() {
                true => {
                    log::info!("Using checkers from {}", &uri);
                    read_checks_from_path(&uri, vec![])
                }
                false => {
                    log::warn!("check-config.toml does not exists.");
                    log::warn!("Trying pyproject.toml.");
                    let uri = parse_path_str_to_uri("pyproject.toml").expect("valid path");
                    match std::path::Path::new(uri.path()).exists() {
                        true => {
                            log::info!("Using checkers from {}", &uri);
                            read_checks_from_path(&uri, vec!["tool", "check-config"])
                        }
                        false => {
                            log::error!("⚠️ No path specified and default paths are not found, so we ran out of options to load the config");
                            return ExitCode::from(ExitStatus::Error);
                        }
                    }
                }
            }
        }
    };

    log::info!("Fix: {}", &cli.fix);

    if cli.list_checkers {
        log::error!("List of checks (type, location of definition, file to check)");
        checks.iter().for_each(|check| {
            log::error!(
                "⬜ {} - {} - {}",
                check.generic_check().file_with_checks.short_url_str(),
                check
                    .generic_check()
                    .file_to_check
                    .as_os_str()
                    .to_string_lossy(),
                check.check_type(),
            )
        });
        return ExitCode::from(ExitStatus::Success);
    }

    let (action_count, success_count) = run_checks(&checks, cli.fix);

    log::warn!("{success_count} checks successful.");
    if action_count > 0 {
        // note: error level is used to always show this message, also with the lowest verbose level
        if action_count == 1 {
            log::error!("🪛  There is 1 violation to fix.",);
        } else {
            log::error!("🪛  There are {action_count} violations to fix.",);
        }
        ExitCode::from(ExitStatus::Failure)
    } else {
        // note: error level is used to always show this message, also with the lowest verbose level
        log::error!("🥇 No violations found.");
        ExitCode::from(ExitStatus::Success)
    }
}

pub(crate) fn run_checks(checks: &Vec<Box<dyn Check>>, fix: bool) -> (i32, i32) {
    let mut action_count = 0;
    let mut success_count = 0;

    for check in checks {
        let result = if fix { check.fix() } else { check.check() };
        match result {
            Err(_) => {
                log::error!("⚠ There was an error fixing files.");
                return (0, 0);
            }
            Ok(Action::None) => success_count += 1,
            Ok(_action) => {
                action_count += 1;
            }
        };
    }

    (action_count, success_count)
}
