use check_config::cli;
use std::process::ExitCode;
pub(crate) fn main() -> ExitCode {
    cli::cli()
}
