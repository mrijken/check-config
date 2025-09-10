use crate::checkers::package::package_present::{
    read_installer_from_check_table, read_package_from_check_table, Installer,
};

use super::super::{
    base::{Action, Check, CheckConstructor, CheckDefinitionError, CheckError},
    GenericCheck,
};

#[derive(Debug)]
pub(crate) struct PackageAbsent {
    generic_check: GenericCheck,
    installer: Installer,
    package: String,
}

impl CheckConstructor for PackageAbsent {
    type Output = Self;

    fn from_check_table(
        generic_check: GenericCheck,
        value: toml_edit::Table,
    ) -> Result<Self::Output, CheckDefinitionError> {
        let installer = read_installer_from_check_table(&value)?;
        let package = read_package_from_check_table(&value)?;
        Ok(Self {
            generic_check,
            installer,
            package,
        })
    }
}

impl Check for PackageAbsent {
    fn check_type(&self) -> String {
        "package_absent".to_string()
    }

    fn generic_check(&self) -> &GenericCheck {
        &self.generic_check
    }

    fn get_action(&self) -> Result<Action, CheckError> {
        match self.generic_check().file_to_check().exists() {
            false => Ok(Action::UninstallPackage {
                installer: self.installer.clone(),
                package: self.package.clone(),
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
