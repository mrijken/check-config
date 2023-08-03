use std::path::PathBuf;

mod checkers;
mod file_types;

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

fn main() {
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

    for check in checks {
        if cli.fix {
            check.fix();
        } else {
            check.check();
        }
    }
}
