use std::path::PathBuf;

mod file_types;
use similar::TextDiff;
mod checkers;
use checkers::base::Action;

use clap::Parser;

use crate::checkers::read_checks_from_path;

/// Config Checker will check and optional fix your config files based on checkers defined in a toml file.
/// It can check ini, toml, yaml, json and plain text files.
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

fn main() -> Result<(), String> {
    simple_logger::init().unwrap();
    log::info!("Starting config-checker");

    let cli = Cli::parse();

    let checkers_path = &PathBuf::from(&cli.path);

    log::info!("Using checkers from {}", &checkers_path.to_string_lossy());
    log::info!("Fix: {}", &cli.fix);

    let checks = read_checks_from_path(checkers_path)?;

    let mut is_all_ok = true;

    for check in checks {
        match check.check() {
            Ok(ist_and_soll) => match ist_and_soll.action() {
                Action::None => {
                    log::info!(
                        "✅ {} - {} - {}",
                        check.checkers_path().to_string_lossy(),
                        check.config_path().to_string_lossy(),
                        check.check_type(),
                    );
                }
                Action::RemoveFile => {
                    log::error!(
                        "❌ {} - {} - {} - file is present",
                        check.checkers_path().to_string_lossy(),
                        check.config_path().to_string_lossy(),
                        check.check_type(),
                    );
                    is_all_ok = false;
                }
                Action::SetContents => {
                    log::error!(
                        "❌ {} - {} - {} - diff",
                        check.checkers_path().to_string_lossy(),
                        check.config_path().to_string_lossy(),
                        check.check_type(),
                    );
                    log::info!(
                        "{}",
                        TextDiff::from_lines(ist_and_soll.ist(), ist_and_soll.soll())
                            .unified_diff()
                    );
                    is_all_ok = false;
                }
            },
            Err(e) => {
                log::error!(
                    "Error: {} {} {} {}",
                    e,
                    check.checkers_path().to_string_lossy(),
                    check.config_path().to_string_lossy(),
                    check.check_type(),
                );
                is_all_ok = false;
            }
        }
        if !is_all_ok && cli.fix {
            log::info!("call fix");
            if check.fix().is_err() {
                panic!("Fix failed");
            }
            is_all_ok = true;
        }
    }

    if is_all_ok {
        Ok(())
    } else {
        Err("One or more errors occured".to_string())
    }
}
