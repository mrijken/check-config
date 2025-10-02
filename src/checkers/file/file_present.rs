use std::fs::Permissions;
#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::PermissionsExt;

use regex::Regex;

use crate::checkers::file::{FileCheck, get_option_string_value_from_checktable};

use super::super::{
    GenericChecker,
    base::{CheckConstructor, CheckDefinitionError, CheckError, Checker},
};

#[derive(Debug)]
pub(crate) struct FilePresent {
    file_check: FileCheck,
    permissions: Option<std::fs::Permissions>,
    placeholder: Option<String>,
    regex: Option<Regex>,
}

pub(crate) fn get_permissions_from_checktable(
    check_table: &toml_edit::Table,
) -> Result<Option<Permissions>, CheckDefinitionError> {
    if let Some(permissions) = check_table.get("permissions") {
        #[cfg(target_os = "windows")]
        {
            Ok(None)
        }

        #[cfg(not(target_os = "windows"))]
        match permissions.as_str() {
            None => Err(CheckDefinitionError::InvalidDefinition(
                "permissions is not a string".into(),
            )),
            Some(permissions) => match u32::from_str_radix(permissions, 8) {
                Err(_) => Err(CheckDefinitionError::InvalidDefinition(
                    "permission can not be converted to an octal mode".into(),
                )),
                Ok(mode) => {
                    if mode > 0o777 {
                        return Err(CheckDefinitionError::InvalidDefinition(
                            "permission is not a valid mode".into(),
                        ));
                    }
                    Ok(Some(std::fs::Permissions::from_mode(mode)))
                }
            },
        }
    } else {
        Ok(None)
    }
}

//[[file_present]]
// file = "file"
// placeholder = "placeholder"  # optional
// regex = "[0-9]*"  # optional
// permissions = "644"  # optional
pub(crate) fn get_regex_from_checktable(
    check_table: &toml_edit::Table,
) -> Result<Option<Regex>, CheckDefinitionError> {
    match get_option_string_value_from_checktable(check_table, "regex") {
        Err(err) => Err(err),
        Ok(None) => Ok(None),
        Ok(Some(regex)) => match Regex::new(regex.as_str()) {
            Ok(r) => Ok(Some(r)),
            Err(_) => Err(CheckDefinitionError::InvalidDefinition(format!(
                "regex ({regex}) is not a valid regex"
            ))),
        },
    }
}

impl CheckConstructor for FilePresent {
    type Output = Self;

    fn from_check_table(
        generic_check: GenericChecker,
        value: toml_edit::Table,
    ) -> Result<Self::Output, CheckDefinitionError> {
        let file_check = FileCheck::from_check_table(generic_check, &value)?;

        let permissions = get_permissions_from_checktable(&value)?;

        let placeholder = get_option_string_value_from_checktable(&value, "placeholder")?;

        let regex = get_regex_from_checktable(&value)?;
        Ok(Self {
            file_check,
            permissions,
            placeholder,
            regex,
        })
    }
}
impl Checker for FilePresent {
    fn checker_type(&self) -> String {
        "file_present".to_string()
    }

    fn generic_checker(&self) -> &GenericChecker {
        &self.file_check.generic_check
    }
    fn checker_object(&self) -> String {
        self.file_check.check_object()
    }
    fn check_(&self, fix: bool) -> Result<crate::checkers::base::CheckResult, CheckError> {
        self.file_check.conclude_check_file_exists(
            self,
            self.placeholder.clone(),
            self.permissions.clone(),
            self.regex.clone(),
            fix,
        )
    }
}

#[cfg(test)]
mod tests {

    use std::fs::write;

    use crate::checkers::{base::CheckResult, test_helpers};

    use super::*;

    use tempfile::tempdir;

    fn get_file_present_check_with_result(
        placeholder: Option<String>,
        permissions: Option<String>,
        regex: Option<String>,
    ) -> (Result<FilePresent, CheckDefinitionError>, tempfile::TempDir) {
        let generic_check = test_helpers::get_generic_check();

        let mut check_table = toml_edit::Table::new();
        let dir = tempdir().unwrap();
        let file_to_check = dir.path().join("file_to_check");
        check_table.insert("file", file_to_check.to_string_lossy().to_string().into());

        if let Some(placeholder) = placeholder {
            check_table.insert("placeholder", placeholder.into());
        }

        if let Some(permissions) = permissions {
            check_table.insert("permissions", permissions.into());
        }

        if let Some(regex) = regex {
            check_table.insert("regex", regex.into());
        }

        (
            FilePresent::from_check_table(generic_check, check_table),
            dir,
        )
    }

    fn get_file_present_check(
        placeholder: Option<String>,
        permissions: Option<String>,
        regex: Option<String>,
    ) -> (FilePresent, tempfile::TempDir) {
        let (file_present_with_result, tempdir) =
            get_file_present_check_with_result(placeholder, permissions, regex);

        (file_present_with_result.unwrap(), tempdir)
    }
    #[test]
    fn test_file_present() {
        let (file_present_check, _tempdir) = get_file_present_check(None, None, None);

        assert_eq!(
            file_present_check.check_(false).unwrap(),
            CheckResult::FixNeeded("create file".into())
        );

        assert_eq!(
            file_present_check.check_(true).unwrap(),
            CheckResult::FixExecuted("create file".into())
        );
        assert_eq!(
            file_present_check.check_(false).unwrap(),
            CheckResult::NoFixNeeded
        );
    }

    #[test]
    fn test_file_present_with_placeholder() {
        let (file_present_check, _tempdir) =
            get_file_present_check(Some("placeholder".into()), None, None);

        assert_eq!(
            file_present_check.check_(false).unwrap(),
            CheckResult::FixNeeded("create file\nset contents to placeholder".into())
        );

        assert_eq!(
            file_present_check.check_(true).unwrap(),
            CheckResult::FixExecuted("create file\nset contents to placeholder".into())
        );
        assert_eq!(
            file_present_check.check_(false).unwrap(),
            CheckResult::NoFixNeeded
        );
    }

    #[test]
    fn test_file_present_with_permissions() {
        let (file_present_check, _tempdir) = get_file_present_check(None, Some("666".into()), None);

        assert_eq!(
            file_present_check.check_(false).unwrap(),
            CheckResult::FixNeeded("create file\nfix permissions to 666".into())
        );

        assert_eq!(
            file_present_check.check_(true).unwrap(),
            CheckResult::FixExecuted("create file\nfix permissions to 666".into())
        );

        assert_eq!(
            file_present_check.check_(false).unwrap(),
            CheckResult::NoFixNeeded
        );
    }

    #[test]
    fn test_file_present_with_regex() {
        let file_present_error =
            get_file_present_check_with_result(None, None, Some("^[0-9]{1,3$".into()))
                .0
                .expect_err("must give error");

        assert_eq!(
            file_present_error,
            CheckDefinitionError::InvalidDefinition(
                "regex (^[0-9]{1,3$) is not a valid regex".into()
            )
        );
        let (file_present_check, _tempdir) =
            get_file_present_check(None, None, Some("[0-9]{1,3}".into()));

        assert_eq!(
            file_present_check.check_(false).unwrap(),
            CheckResult::FixNeeded("create file\nfix content to match regex \"[0-9]{1,3}\"".into())
        );

        let _ = write(
            file_present_check.file_check.file_to_check.as_ref().clone(),
            "bla",
        );

        assert_eq!(
            file_present_check.check_(false).unwrap(),
            CheckResult::FixNeeded("fix content to match regex \"[0-9]{1,3}\"".into())
        );

        let _ = write(
            file_present_check.file_check.file_to_check.as_ref().clone(),
            "129",
        );

        assert_eq!(
            file_present_check.check_(false).unwrap(),
            CheckResult::NoFixNeeded
        );
    }
}
