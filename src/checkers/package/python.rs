use crate::checkers::{
    base::CheckError,
    package::{
        Installer, Package,
        command::{run_command_stream, run_command_stream_capture_stdout},
    },
};

pub(crate) struct UV;

impl Installer for UV {
    fn install(package: &Package) -> Result<(), CheckError> {
        let package_specifier = if let Some(version) = &package.version {
            format!("{package}=={version}", package = &package.name)
        } else {
            package.name.to_owned()
        };

        run_command_stream(
            "uv",
            vec!["tool", "install", package_specifier.as_str()].as_ref(),
        )
    }

    fn uninstall(package: &Package) -> Result<(), CheckError> {
        run_command_stream(
            "uv",
            vec!["tool", "uninstall", package.name.as_str()].as_ref(),
        )
    }

    fn is_installed(package: &Package) -> Result<bool, CheckError> {
        let stdout = run_command_stream_capture_stdout(
            "uv",
            vec!["tool", "uninstall", package.name.as_str()].as_ref(),
        )?;

        let packages: Vec<&str> = stdout
            .lines()
            .filter(|line| line.starts_with(format!("{package} ", package = package.name).as_str()))
            .collect();

        Ok(if packages.len() != 1 {
            false
        } else if let Some(version) = package.version.as_ref() {
            packages
                .first()
                .expect("1 item present")
                .split_once(" ")
                .expect("space is present")
                .1
                == version
        } else {
            true
        })
    }
}
