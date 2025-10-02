use std::io::Write;
use std::process::ExitCode;

use clap::Parser;

use crate::checkers::{RelativeUrl, base::Checker};

use super::checkers::read_checks_from_path;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ExitStatus {
    /// Reading checks was succesful and there are no checks to fix
    Success,
    /// Reading checks was succesul and there are checks to fix
    Failure,
    /// Reading checks was faild or executing fixes was failed
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
    /// Path or URL to the root checkers file in toml format
    /// Defaults (in order of precedence):
    /// - check-config.toml
    /// - pyproject.toml with a tool.check-config key
    #[arg(short, long, env = "CHECK_CONFIG_PATH", verbatim_doc_comment)]
    path: Option<String>,

    /// Try to fix the config
    #[arg(long, default_value = "false")]
    fix: bool,

    /// List all checks. Checks are not executed.
    #[arg(short, long, default_value = "false")]
    list_checkers: bool,

    /// Execute the checkers with one of the specified tags
    #[arg(long, value_delimiter = ',', env = "CHECK_CONFIG_ANY_TAGS")]
    any_tags: Vec<String>,

    /// Execute the checkers with all of the specified tags
    #[arg(long, value_delimiter = ',', env = "CHECK_CONFIG_ALL_TAGS")]
    all_tags: Vec<String>,

    /// Do not execute the checkers with any of the specified tags.
    #[arg(long, value_delimiter = ',', env = "CHECK_CONFIG_SKIP_TAGS")]
    skip_tags: Vec<String>,

    /// Create missing directories
    #[arg(short, long, default_value = "false", env = "CHECK_CONFIG_CREATE_DIRS")]
    create_missing_directories: bool,

    // -v s
    // -vv show all
    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

pub(crate) fn filter_checks(
    checker_tags: &[String],
    any_tags: &[String],
    all_tags: &[String],
    skip_tags: &[String],
) -> bool {
    // At least one must match
    if !any_tags.is_empty() && !any_tags.iter().any(|t| checker_tags.contains(t)) {
        return false;
    }

    // All must match
    if !all_tags.is_empty() && !all_tags.iter().all(|t| checker_tags.contains(t)) {
        return false;
    }

    // None must match
    if !skip_tags.is_empty() && skip_tags.iter().any(|t| checker_tags.contains(t)) {
        return false;
    }

    true
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

    let mut checks = match cli.path {
        Some(path_str) => match parse_path_str_to_uri(path_str.as_str()) {
            Some(uri) => {
                log::info!("Using checkers from {}", &uri.short_url_str());
                read_checks_from_path(&uri, vec![])
            }
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
                            log::error!(
                                "⚠️ No path specified and default paths are not found, so we ran out of options to load the config"
                            );
                            return ExitCode::from(ExitStatus::Error);
                        }
                    }
                }
            }
        }
    };

    log::info!("Fix: {}", &cli.fix);

    if cli.list_checkers {
        log::error!("List of checks (type, location of definition, file to check, tags)");
        checks.iter().for_each(|check| {
            let enabled = filter_checks(
                &check.generic_checker().tags,
                &cli.any_tags,
                &cli.all_tags,
                &cli.skip_tags,
            );

            check.list_checker(enabled);
        });
        return ExitCode::from(ExitStatus::Success);
    }

    // log::info!(
    //     "☰ Restricting checkers which have tags which all are part of: {:?}",
    //     restricted_tags,
    // );

    checks.retain(|check| {
        filter_checks(
            &check.generic_checker().tags,
            &cli.any_tags,
            &cli.all_tags,
            &cli.skip_tags,
        )
    });

    ExitCode::from(run_checks(&checks, cli.fix))
}

pub(crate) fn run_checks(checks: &Vec<Box<dyn Checker>>, fix: bool) -> ExitStatus {
    let mut fix_needed_count = 0;
    let mut fix_executed_count = 0;
    let mut no_fix_needed_count = 0;
    let mut error_count = 0;

    for check in checks {
        let fix = check.generic_checker().fixable && fix;
        let result = check.check(fix);
        match result {
            crate::checkers::base::CheckResult::NoFixNeeded => no_fix_needed_count += 1,
            crate::checkers::base::CheckResult::FixExecuted(_) => fix_executed_count += 1,
            crate::checkers::base::CheckResult::FixNeeded(_) => fix_needed_count += 1,
            crate::checkers::base::CheckResult::Error(_) => error_count += 1,
        };
    }

    log::warn!("{checks} checks found", checks = checks.len());
    if fix {
        log::warn!("✅ {fix_executed_count} checks fixed");
        log::warn!("✅ {no_fix_needed_count} checks did not a fix");
    }

    match fix_needed_count {
        0 => log::error!("🥇 No violations found."),

        1 => log::error!("🪛 There is 1 violation to fix.",),
        _ => log::error!("🪛 There are {fix_needed_count} violations to fix.",),
    }

    match error_count {
        0 => (),

        1 => log::error!("🚨 There was 1 error executing a fix.",),
        _ => log::error!("🚨 There are {error_count} errors executing a fix.",),
    }
    if error_count > 0 {
        ExitStatus::Error
    } else if fix_needed_count > 0 {
        ExitStatus::Failure
    } else {
        ExitStatus::Success
    }
}
