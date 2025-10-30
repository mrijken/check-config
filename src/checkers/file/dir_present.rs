use std::fs;
#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::PermissionsExt;

use crate::{
    checkers::{
        base::CheckResult,
        file::{file_present::get_permissions_from_checktable, get_string_value_from_checktable},
    },
    uri::WritablePath,
};

use super::super::{
    GenericChecker,
    base::{CheckConstructor, CheckDefinitionError, CheckError, Checker},
};

#[derive(Debug)]
pub(crate) struct DirPresent {
    generic_check: GenericChecker,
    dir: WritablePath,
    permissions: Option<std::fs::Permissions>,
}

//[[dir_present]]
// dir = "dir"
// permissions = "755"  # optional

impl CheckConstructor for DirPresent {
    type Output = Self;

    fn from_check_table(
        generic_check: GenericChecker,
        check_table: toml_edit::Table,
    ) -> Result<Self::Output, CheckDefinitionError> {
        let dir = WritablePath::from_string(
            get_string_value_from_checktable(&check_table, "dir")?.as_str(),
        )
        .map_err(|_| CheckDefinitionError::InvalidDefinition("invalid destination path".into()))?;

        let permissions = get_permissions_from_checktable(&check_table)?;

        Ok(Self {
            generic_check,
            dir,
            permissions,
        })
    }
}
impl Checker for DirPresent {
    fn checker_type(&self) -> String {
        "dir_present".to_string()
    }

    fn generic_checker(&self) -> &GenericChecker {
        &self.generic_check
    }
    fn checker_object(&self) -> String {
        self.dir.to_string()
    }
    fn check_(&self, fix: bool) -> Result<crate::checkers::base::CheckResult, CheckError> {
        let mut action_messages: Vec<String> = vec![];

        let create_dir = !self.dir.exists();

        if create_dir {
            action_messages.push("create dir".into());
        }

        let fix_permissions = if let Some(permissions) = self.permissions.clone() {
            #[cfg(target_os = "windows")]
            {
                false
            }

            #[cfg(not(target_os = "windows"))]
            {
                if create_dir {
                    true
                } else {
                    let current_permissions = match self.dir.as_ref().metadata() {
                        Err(_) => {
                            return Err(CheckError::PermissionsNotAccessable);
                        }
                        Ok(metadata) => metadata.permissions(),
                    };

                    // we only check for the last 3 octal digits

                    (current_permissions.mode() & 0o777) != (permissions.mode() & 0o777)
                }
            }
        } else {
            false
        };

        #[cfg(not(target_os = "windows"))]
        if fix_permissions {
            action_messages.push(format!(
                "fix permissions to {:o}",
                self.permissions.clone().unwrap().to_owned().mode()
            ));
        }

        let action_message = action_messages.join("\n");

        let check_result = match (create_dir || fix_permissions, fix) {
            (false, _) => CheckResult::NoFixNeeded,
            (true, false) => CheckResult::FixNeeded(action_message),
            (true, true) => {
                fs::create_dir_all(self.dir.as_ref())?;
                if fix_permissions {
                    fs::set_permissions(self.dir.as_ref(), self.permissions.clone().unwrap())?;
                }
                CheckResult::FixExecuted(action_message)
            }
        };

        Ok(check_result)
    }
}

#[cfg(test)]
mod tests {

    use crate::checkers::{base::CheckResult, test_helpers};

    use super::*;

    use tempfile::tempdir;

    fn get_dir_present_check_present_check_with_result(
        permissions: Option<String>,
    ) -> (Result<DirPresent, CheckDefinitionError>, tempfile::TempDir) {
        let generic_check = test_helpers::get_generic_check();

        let mut check_table = toml_edit::Table::new();
        let dir = tempdir().unwrap();
        let dir_to_check = dir.path().join("dir_to_check");
        check_table.insert("dir", dir_to_check.to_string_lossy().to_string().into());

        if let Some(permissions) = permissions {
            check_table.insert("permissions", permissions.into());
        }

        (
            DirPresent::from_check_table(generic_check, check_table),
            dir,
        )
    }

    fn get_dir_present_check(permissions: Option<String>) -> (DirPresent, tempfile::TempDir) {
        let (dir_present_with_result, tempdir) =
            get_dir_present_check_present_check_with_result(permissions);

        (dir_present_with_result.unwrap(), tempdir)
    }
    #[test]
    fn test_dir_present() {
        let (dir_present_check, _tempdir) = get_dir_present_check(None);

        assert_eq!(
            dir_present_check.check_(false).unwrap(),
            CheckResult::FixNeeded("create dir".into())
        );

        assert_eq!(
            dir_present_check.check_(true).unwrap(),
            CheckResult::FixExecuted("create dir".into())
        );
        assert_eq!(
            dir_present_check.check_(false).unwrap(),
            CheckResult::NoFixNeeded
        );
    }

    #[test]
    fn test_dir_present_with_permissions() {
        let (dir_present_check, _tempdir) = get_dir_present_check(Some("777".into()));

        assert_eq!(
            dir_present_check.check_(false).unwrap(),
            CheckResult::FixNeeded("create dir\nfix permissions to 777".into())
        );

        assert_eq!(
            dir_present_check.check_(true).unwrap(),
            CheckResult::FixExecuted("create dir\nfix permissions to 777".into())
        );

        assert_eq!(
            dir_present_check.check_(false).unwrap(),
            CheckResult::NoFixNeeded
        );
    }
}
