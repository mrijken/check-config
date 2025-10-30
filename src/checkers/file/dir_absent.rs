use std::fs;

use crate::{
    checkers::{base::CheckResult, file::get_string_value_from_checktable},
    uri::WritablePath,
};

use super::super::{
    GenericChecker,
    base::{CheckConstructor, CheckDefinitionError, CheckError, Checker},
};

#[derive(Debug)]
pub(crate) struct DirAbsent {
    generic_check: GenericChecker,
    dir: WritablePath,
}

//[[dir_absent]]
// dir = "dir"

impl CheckConstructor for DirAbsent {
    type Output = Self;

    fn from_check_table(
        generic_check: GenericChecker,
        check_table: toml_edit::Table,
    ) -> Result<Self::Output, CheckDefinitionError> {
        let dir = WritablePath::from_string(
            get_string_value_from_checktable(&check_table, "dir")?.as_str(),
        )
        .map_err(|_| CheckDefinitionError::InvalidDefinition("invalid destination path".into()))?;

        Ok(Self { generic_check, dir })
    }
}
impl Checker for DirAbsent {
    fn checker_type(&self) -> String {
        "dir_absent".to_string()
    }

    fn generic_checker(&self) -> &GenericChecker {
        &self.generic_check
    }
    fn checker_object(&self) -> String {
        self.dir.to_string()
    }
    fn check_(&self, fix: bool) -> Result<crate::checkers::base::CheckResult, CheckError> {
        let mut action_messages: Vec<String> = vec![];

        let remove_dir = self.dir.exists();

        if remove_dir {
            action_messages.push("remove dir".into());
        }

        let action_message = action_messages.join("\n");

        let check_result = match (remove_dir, fix) {
            (false, _) => CheckResult::NoFixNeeded,
            (true, false) => CheckResult::FixNeeded(action_message),
            (true, true) => {
                fs::remove_dir_all(self.dir.as_ref())?;
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

    fn get_dir_absent_check_present_check_with_result()
    -> (Result<DirAbsent, CheckDefinitionError>, tempfile::TempDir) {
        let generic_check = test_helpers::get_generic_check();

        let mut check_table = toml_edit::Table::new();
        let dir = tempdir().unwrap();
        let dir_to_check = dir.path().join("dir_to_check");
        check_table.insert("dir", dir_to_check.to_string_lossy().to_string().into());

        (DirAbsent::from_check_table(generic_check, check_table), dir)
    }

    fn get_dir_absent_check() -> (DirAbsent, tempfile::TempDir) {
        let (dir_absent_with_result, tempdir) = get_dir_absent_check_present_check_with_result();

        (dir_absent_with_result.unwrap(), tempdir)
    }

    #[test]
    fn test_dir_absent() {
        let (dir_absent_check, _tempdir) = get_dir_absent_check();

        assert_eq!(
            dir_absent_check.check_(false).unwrap(),
            CheckResult::NoFixNeeded
        );

        fs::create_dir(dir_absent_check.dir.as_ref()).unwrap();

        assert_eq!(
            dir_absent_check.check_(false).unwrap(),
            CheckResult::FixNeeded("remove dir".into())
        );
        assert_eq!(
            dir_absent_check.check_(true).unwrap(),
            CheckResult::FixExecuted("remove dir".into())
        );

        assert_eq!(
            dir_absent_check.check_(false).unwrap(),
            CheckResult::NoFixNeeded
        );
    }
}
