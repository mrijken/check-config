use derive_more::Display;

use crate::checkers::{
    base::{CheckDefinitionError, CheckError},
    file::{get_option_string_value_from_checktable, get_string_value_from_checktable},
};

mod command;
pub(crate) mod custom;
pub(crate) mod package_absent;
pub(crate) mod package_present;
pub(crate) mod python;
pub(crate) mod rust;

// inspiration: https://github.com/mason-org/mason.nvim/tree/ad7146aa61dcaeb54fa900144d768f040090bff0/lua/mason-core/installer/managers

#[derive(Clone, Debug, PartialEq, Display)]
#[display("custom: {name}")]
pub(crate) struct CustomInstaller {
    name: String,
    install_command: Option<String>,
    version_command: String,
    uninstall_command: Option<String>,
    version: String,
}

#[derive(Clone, Debug, PartialEq, Display)]
pub(crate) enum PackageType {
    #[display("python: {name}", name=_0.name)]
    Python(Package),
    #[display("crate: {name}", name=_0.name)]
    Rust(Package),
    #[display("github: {name}", name=_0.name)]
    GithubRelease(Package),
    #[display("gitlab: {name}", name=_0.name)]
    GitlabRelease(Package),
    #[display("command: {name}", name=_0.name)]
    Custom(CustomInstaller),
}

impl PackageType {
    // TODO: add installer to Package to differentiate between installer (ie uv or pipx for
    // PythonPackage)

    // Install a package
    pub(crate) fn install(&self) -> Result<(), CheckError> {
        match self {
            PackageType::Python(package) => python::UV::install(package),
            PackageType::Rust(package) => rust::Cargo::install(package),
            PackageType::Custom(package) => custom::install(package),

            _ => todo!(),
        }
    }

    pub(crate) fn uninstall(&self) -> Result<(), CheckError> {
        match self {
            PackageType::Python(package) => python::UV::uninstall(package),
            PackageType::Rust(package) => rust::Cargo::uninstall(package),
            PackageType::Custom(package) => custom::uninstall(package),
            _ => todo!(),
        }
    }

    // is the package installed with the given version.
    // if no version is given, return true if the package is installed
    pub(crate) fn is_installed(&self) -> Result<bool, CheckError> {
        match self {
            PackageType::Python(package) => python::UV::is_installed(package),
            PackageType::Rust(package) => rust::Cargo::is_installed(package),
            PackageType::Custom(package) => custom::is_installed(package),
            _ => todo!(),
        }
    }
    // if no version is specified, we will upgrade the package to the latest version.
    // return True when there is possible or certain a newer version available
    // Note: possible is the case when the package manager can not report a newer version without
    // installing it
    pub(crate) fn is_upgradable(&self) -> Result<bool, CheckError> {
        match self {
            PackageType::Python(package) => python::UV::is_upgradable(package),
            PackageType::Rust(package) => rust::Cargo::is_upgradable(package),
            PackageType::Custom(package) => custom::is_upgradable(package),
            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Display)]
#[display("{name}")] // TODO: add version when not none
pub(crate) struct Package {
    pub(crate) name: String,
    pub(crate) version: Option<String>,
    pub(crate) bins: Vec<String>,
    // platform: Platform
    // arch: Arch
}

pub(crate) trait Installer {
    fn install(package: &Package) -> Result<(), CheckError>;
    fn uninstall(package: &Package) -> Result<(), CheckError>;
    fn is_installed(package: &Package) -> Result<bool, CheckError>;
    fn is_upgradable(package: &Package) -> Result<bool, CheckError>;
}

pub(crate) fn read_package_from_check_table(
    value: &toml_edit::Table,
) -> Result<PackageType, CheckDefinitionError> {
    let package_type = match value.get("type") {
        None => Err(CheckDefinitionError::InvalidDefinition(
            "No type present".into(),
        )),
        Some(package_type) => match package_type.as_str() {
            None => Err(CheckDefinitionError::InvalidDefinition(
                "type is not a string".into(),
            )),
            Some(package_type) => Ok(package_type.to_lowercase()),
        },
    }?;

    if package_type == "custom" {
        return Ok(PackageType::Custom(CustomInstaller {
            name: get_string_value_from_checktable(value, "package")?,
            install_command: get_option_string_value_from_checktable(value, "install_command")?,
            uninstall_command: get_option_string_value_from_checktable(value, "uninstall_command")?,
            version_command: get_string_value_from_checktable(value, "version_command")?,
            version: get_string_value_from_checktable(value, "version")?,
        }));
    }

    let package_name = read_package_name_from_check_table(value)?;

    let package_version = read_optional_version_from_check_table(value)?;

    let package = Package {
        name: package_name,
        version: package_version,
        bins: vec![],
    };

    match package_type.to_lowercase().as_str() {
        "python" => Ok(PackageType::Python(package)),
        "rust" => Ok(PackageType::Rust(package)),
        _ => Err(CheckDefinitionError::InvalidDefinition(format!(
            "unknown package_type {package_type}"
        ))),
    }
}

pub(crate) fn read_package_name_from_check_table(
    value: &toml_edit::Table,
) -> Result<String, CheckDefinitionError> {
    match value.get("package") {
        None => Err(CheckDefinitionError::InvalidDefinition(
            "No package present".into(),
        )),
        Some(package) => match package.as_str() {
            None => Err(CheckDefinitionError::InvalidDefinition(
                "package is not a string".into(),
            )),
            Some(package) => Ok(package.to_owned()),
        },
    }
}

pub(crate) fn read_optional_version_from_check_table(
    value: &toml_edit::Table,
) -> Result<Option<String>, CheckDefinitionError> {
    match value.get("version") {
        None => Ok(None),
        Some(package) => match package.as_str() {
            None => Err(CheckDefinitionError::InvalidDefinition(
                "version is not a string".into(),
            )),
            Some(package) => Ok(Some(package.to_owned())),
        },
    }
}
