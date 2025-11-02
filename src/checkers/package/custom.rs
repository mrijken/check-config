use crate::checkers::{
    base::CheckError,
    package::{
        CustomInstaller,
        command::{run_command_stream, run_command_stream_capture_stdout},
    },
};

fn make_command_and_args(shell_command: &str) -> (&str, Vec<&str>) {
    if cfg!(target_os = "windows") {
        ("cmd", vec!["/C", shell_command])
    } else {
        ("sh", vec!["-c", shell_command])
    }
}

pub fn install(package: &CustomInstaller) -> Result<(), CheckError> {
    let shell_command = package.install_command.as_ref().ok_or(CheckError::String(
        "install_command is not specified".to_string(),
    ))?;
    let (command, args) = make_command_and_args(shell_command);
    run_command_stream(command, &args)
}

pub fn uninstall(package: &CustomInstaller) -> Result<(), CheckError> {
    let shell_command = package
        .uninstall_command
        .as_ref()
        .ok_or(CheckError::String(
            "uninstall_command is not specified".to_string(),
        ))?;
    let (command, args) = make_command_and_args(shell_command);

    run_command_stream(command, &args)
}

pub fn is_installed(package: &CustomInstaller) -> Result<bool, CheckError> {
    let shell_command = &package.version_command;
    let (command, args) = make_command_and_args(shell_command);

    let stdout = run_command_stream_capture_stdout(command, &args)?;

    Ok(stdout.contains(package.version.as_str()))
}

pub fn is_upgradable(_package: &CustomInstaller) -> Result<bool, CheckError> {
    Ok(true)
}
