use crate::checkers::base::CheckResult;
use crate::checkers::file::FileCheck;

use super::super::GenericChecker;
use super::super::base::{CheckConstructor, CheckDefinitionError, CheckError, Checker};

#[derive(Debug, Clone)]
pub(crate) struct FileAbsent {
    file_check: FileCheck,
}

// [[file_absent]]
// file = "file"
impl CheckConstructor for FileAbsent {
    type Output = Self;

    fn from_check_table(
        generic_check: GenericChecker,
        check_table: toml_edit::Table,
    ) -> Result<Self::Output, CheckDefinitionError> {
        let file_check = FileCheck::from_check_table(generic_check, &check_table)?;
        Ok(Self { file_check })
    }
}
impl Checker for FileAbsent {
    fn checker_type(&self) -> String {
        "file_absent".to_string()
    }

    fn checker_object(&self) -> String {
        self.file_check.check_object()
    }

    fn generic_checker(&self) -> &GenericChecker {
        &self.file_check.generic_check
    }

    fn check_(&self, fix: bool) -> Result<CheckResult, CheckError> {
        self.file_check.conclude_check_with_remove(self, fix)
    }
}

#[cfg(test)]
mod tests {

    use std::fs::File;

    use crate::checkers::test_helpers;

    use super::*;

    use tempfile::{TempDir, tempdir};

    fn get_file_absent_check() -> (FileAbsent, TempDir) {
        let generic_check = test_helpers::get_generic_check();

        let mut check_table = toml_edit::Table::new();
        let dir = tempdir().unwrap();
        let file_to_check = dir.path().join("file_to_check");
        check_table.insert("file", file_to_check.to_string_lossy().to_string().into());

        (
            FileAbsent::from_check_table(generic_check, check_table).unwrap(),
            dir,
        )
    }

    #[test]
    fn test_file_absent() {
        let (file_absent_check, _tmpdir) = get_file_absent_check();

        assert_eq!(
            file_absent_check.check_(false).unwrap(),
            CheckResult::NoFixNeeded
        );

        File::create(file_absent_check.file_check.file_to_check.as_ref()).expect("file is created");

        assert_eq!(
            file_absent_check.check_(true).unwrap(),
            CheckResult::FixExecuted("remove file".into())
        );

        assert_eq!(
            file_absent_check.check_(false).unwrap(),
            CheckResult::NoFixNeeded
        );
    }
}
