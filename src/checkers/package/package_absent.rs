use crate::checkers::{
    base::{CheckResult, Checker},
    package::{PackageType, read_package_from_check_table},
};

use super::super::{
    GenericChecker,
    base::{CheckConstructor, CheckDefinitionError, CheckError},
};

#[derive(Debug)]
pub(crate) struct PackageAbsent {
    generic_check: GenericChecker,
    package: PackageType,
}

impl CheckConstructor for PackageAbsent {
    type Output = Self;

    fn from_check_table(
        generic_check: GenericChecker,
        value: toml_edit::Table,
    ) -> Result<Self::Output, CheckDefinitionError> {
        let package_type = read_package_from_check_table(&value)?;
        Ok(Self {
            generic_check,
            package: package_type,
        })
    }
}

impl Checker for PackageAbsent {
    fn checker_type(&self) -> String {
        "package_absent".to_string()
    }

    fn generic_checker(&self) -> &GenericChecker {
        &self.generic_check
    }

    fn checker_object(&self) -> String {
        format!("{}", self.package)
    }

    fn check_(&self, fix: bool) -> Result<crate::checkers::base::CheckResult, CheckError> {
        let to_uninstall = self.package.is_installed()?;

        let action_message = if to_uninstall {
            format!("uninstall package {}", self.package)
        } else {
            "".to_string()
        };

        let check_result = match (to_uninstall, fix) {
            (false, _) => CheckResult::NoFixNeeded,
            (true, false) => CheckResult::FixNeeded(action_message),
            (true, true) => {
                self.package.uninstall()?;
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
