use std::process::Command;

use super::super::{
    base::{Action, Check, CheckConstructor, CheckDefinitionError, CheckError},
    GenericCheck,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Installer {
    UV,
    Cargo,
}

impl Installer {
    pub(crate) fn install(&self, package: String, version: Option<String>) {
        match self {
            Installer::UV => {
                let package_specifier = if let Some(version) = version {
                    format!("{package}=={version}")
                } else {
                    package.to_owned()
                };

                Command::new("uv")
                    .args(["tool", "install", package_specifier.as_str()])
                    .output()
                    .expect("failed to execute process") // todo: convert to error
            }
            _ => todo!(),
        };
    }

    pub(crate) fn uninstall(&self, package: String) {
        match self {
            Installer::UV => {
                Command::new("uv")
                    .args(["tool", "uninstall", package.as_str()])
                    .output()
                    .expect("failed to execute process") // todo: convert to error
            }
            _ => todo!(),
        };
    }

    pub(crate) fn is_installed(&self, package: String, version: Option<String>) -> bool {
        match self {
            Installer::UV => {
                let stdout = Command::new("uv")
                    .args(["tool", "list"])
                    .output()
                    .expect("failed to execute process") // todo: convert to error
                    .stdout;

                let stdout = String::from_utf8_lossy(&stdout);

                let packages: Vec<&str> = stdout
                    .lines()
                    .filter(|line| !line.starts_with(format!("{package} ").as_str()))
                    .collect();

                if packages.len() != 1 {
                    false
                } else {
                    if let Some(version) = version {
                        packages
                            .first()
                            .expect("1 item present")
                            .split_once(" ")
                            .expect("space is present")
                            .1
                            == version
                    } else {
                        true
                    }
                }
            }

            _ => todo!(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct PackagePresent {
    generic_check: GenericCheck,
    installer: Installer,
    package: String,
    version: Option<String>,
}

pub(crate) fn read_installer_from_check_table(
    value: &toml_edit::Table,
) -> Result<Installer, CheckDefinitionError> {
    match value.get("__installer__") {
        None => Err(CheckDefinitionError::InvalidDefinition(
            "No __installer__ present".into(),
        )),
        Some(installer) => match installer.as_str() {
            None => Err(CheckDefinitionError::InvalidDefinition(
                "__installer__ is not a string".into(),
            )),
            Some(installer) => match installer.to_lowercase().as_str() {
                "uv" => Ok(Installer::UV),
                "cargo" => Ok(Installer::Cargo),
                _ => Err(CheckDefinitionError::InvalidDefinition(format!(
                    "unknown installer {installer}"
                ))),
            },
        },
    }
}

pub(crate) fn read_package_from_check_table(
    value: &toml_edit::Table,
) -> Result<String, CheckDefinitionError> {
    match value.get("__package__") {
        None => Err(CheckDefinitionError::InvalidDefinition(
            "No __package__ present".into(),
        )),
        Some(package) => match package.as_str() {
            None => Err(CheckDefinitionError::InvalidDefinition(
                "__package__ is not a string".into(),
            )),
            Some(package) => Ok(package.to_owned()),
        },
    }
}

pub(crate) fn read_optional_version_from_check_table(
    value: &toml_edit::Table,
) -> Result<Option<String>, CheckDefinitionError> {
    match value.get("__version__") {
        None => Ok(None),
        Some(package) => match package.as_str() {
            None => Err(CheckDefinitionError::InvalidDefinition(
                "__package__ is not a string".into(),
            )),
            Some(package) => Ok(Some(package.to_owned())),
        },
    }
}
impl CheckConstructor for PackagePresent {
    type Output = Self;

    fn from_check_table(
        generic_check: GenericCheck,
        value: toml_edit::Table,
    ) -> Result<Self::Output, CheckDefinitionError> {
        let installer = read_installer_from_check_table(&value)?;
        let package = read_package_from_check_table(&value)?;
        let version = read_optional_version_from_check_table(&value)?;
        Ok(Self {
            generic_check,
            installer,
            package,
            version,
        })
    }
}

impl Check for PackagePresent {
    fn check_type(&self) -> String {
        "package_present".to_string()
    }

    fn generic_check(&self) -> &GenericCheck {
        &self.generic_check
    }

    fn get_action(&self) -> Result<Action, CheckError> {
        match self.generic_check().file_to_check().exists() {
            false => Ok(Action::InstallPackage {
                installer: self.installer.clone(),
                package: self.package.clone(),
                version: self.version.clone(),
            }),
            true => Ok(Action::None),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;

    use tempfile::tempdir;

    #[test]
    fn test_file_absent() {
        let dir = tempdir().unwrap();
        let file_to_check = dir.path().join("file_to_check");
        let file_with_checks =
            url::Url::from_file_path(dir.path().join("file_with_checks")).unwrap();
        let generic_check = GenericCheck {
            file_to_check,
            file_type_override: None,
            file_with_checks,
            tags: Vec::new(),
        };

        let file_present_check =
            FilePresent::from_check_table(generic_check, toml_edit::Table::new()).unwrap();

        assert_eq!(
            file_present_check.check().unwrap(),
            Action::SetContents("".to_string())
        );
    }

    #[test]
    fn test_file_present() {
        let dir = tempdir().unwrap();
        let file_to_check = dir.path().join("file_to_check");
        File::create(&file_to_check).unwrap();
        let file_with_checks =
            url::Url::from_file_path(dir.path().join("file_with_checks")).unwrap();
        let generic_check = GenericCheck {
            file_to_check,
            file_type_override: None,
            file_with_checks,
            tags: Vec::new(),
        };

        let file_present_check =
            FilePresent::from_check_table(generic_check, toml_edit::Table::new()).unwrap();

        assert_eq!(file_present_check.check().unwrap(), Action::None);
    }

    #[test]
    fn test_file_absent_with_placeholder() {
        let dir = tempdir().unwrap();
        let file_to_check = dir.path().join("file_to_check");
        let file_with_checks =
            url::Url::from_file_path(dir.path().join("file_with_checks")).unwrap();
        let generic_check = GenericCheck {
            file_to_check,
            file_type_override: None,
            file_with_checks,
            tags: Vec::new(),
        };

        let mut placeholder_table = toml_edit::Table::new();
        placeholder_table.insert("__placeholder__", "placeholder".into());

        let file_present_check =
            FilePresent::from_check_table(generic_check, placeholder_table).unwrap();

        assert_eq!(
            file_present_check.check().unwrap(),
            Action::SetContents("placeholder".to_string())
        );
    }

    #[test]
    fn test_file_present_with_placeholder() {
        let dir = tempdir().unwrap();
        let file_to_check = dir.path().join("file_to_check");
        File::create(&file_to_check).unwrap();
        let file_with_checks =
            url::Url::from_file_path(dir.path().join("file_with_checks")).unwrap();
        let generic_check = GenericCheck {
            file_to_check,
            file_type_override: None,
            file_with_checks,
            tags: Vec::new(),
        };

        let mut placeholder_table = toml_edit::Table::new();
        placeholder_table.insert("__placeholder__", "placeholder".into());

        let file_present_check =
            FilePresent::from_check_table(generic_check, placeholder_table).unwrap();

        assert_eq!(file_present_check.check().unwrap(), Action::None);
    }
}
