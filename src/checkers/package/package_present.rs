use crate::checkers::{
    GenericChecker,
    base::{CheckConstructor, CheckDefinitionError, CheckError, CheckResult, Checker},
    package::{PackageType, read_package_from_check_table},
};

#[derive(Debug)]
pub(crate) struct PackagePresent {
    generic_checker: GenericChecker,
    package: PackageType,
}

impl CheckConstructor for PackagePresent {
    type Output = Self;

    fn from_check_table(
        generic_check: GenericChecker,
        check_table: toml_edit::Table,
    ) -> Result<Self::Output, CheckDefinitionError> {
        let package = read_package_from_check_table(&check_table)?;
        Ok(Self {
            generic_checker: generic_check,
            package,
        })
    }
}

impl Checker for PackagePresent {
    fn checker_type(&self) -> String {
        "package_present".to_string()
    }

    fn generic_checker(&self) -> &GenericChecker {
        &self.generic_checker
    }

    fn checker_object(&self) -> String {
        format!("{}", self.package)
    }

    fn check_(&self, fix: bool) -> Result<crate::checkers::base::CheckResult, CheckError> {
        let to_install = !self.package.is_installed()?;
        let try_to_upgrade = self.package.is_upgradable()?;

        let action_message = if to_install {
            format!("install package {}", self.package)
        } else if try_to_upgrade {
            format!("try to upgrade package {} to latest", self.package,)
        } else {
            "".to_string()
        };

        let check_result = match (to_install, fix) {
            (false, _) => CheckResult::NoFixNeeded,
            (true, false) => CheckResult::FixNeeded(action_message),
            (true, true) => {
                self.package.install()?;
                CheckResult::FixExecuted(action_message)
            }
        };

        Ok(check_result)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;

    use tempfile::tempdir;
}
