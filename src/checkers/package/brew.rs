use crate::checkers::{
    base::CheckError,
    package::{
        Installer, Package,
        command::{run_command_stream, run_command_stream_capture_stdout},
    },
};

pub(crate) struct Brew;

impl Installer for Brew {
    fn install(package: &Package) -> Result<(), CheckError> {
        let package_specifier = if let Some(version) = &package.version {
            format!("{package}@{version}", package = package.name)
        } else {
            package.name.to_owned()
        };

        run_command_stream("brew", vec!["install", package_specifier.as_str()].as_ref())
    }

    fn uninstall(package: &Package) -> Result<(), CheckError> {
        run_command_stream("brew", vec!["uninstall", package.name.as_str()].as_ref())
    }

    fn is_upgradable(package: &Package) -> Result<bool, CheckError> {
        // assume every installed package is upgradable, as lots of packages on brew have only
        // latest/stable present
        Brew::is_installed(package)
    }

    fn is_installed(package: &Package) -> Result<bool, CheckError> {
        let stdout = match run_command_stream_capture_stdout(
            "brew",
            vec!["info", package.name.as_str()].as_ref(),
        ) {
            Err(_) => return Ok(false),
            Ok(s) => s,
        };

        Ok(if let Some(version) = package.version.as_ref() {
            stdout.contains(version)
        } else {
            true
        })
    }
}
