use crate::{
    checkers::{base::CheckResult, file::get_string_value_from_checktable},
    uri::{ReadPath, ReadablePath, WritablePath},
};

use super::super::{
    GenericChecker,
    base::{CheckConstructor, CheckDefinitionError, CheckError, Checker},
};

#[derive(Debug)]
pub(crate) struct FileCopied {
    source: ReadablePath,
    destination: WritablePath,
    generic_check: GenericChecker,
}

//[[file_copied]]
// file = "path or url of file to copy"
// destination = "path (including filename) to copy to"
//
// check if file is copied
// if source is a relative path, it's relative to the check file, so the dir
// which contain the file which defines this check.
impl CheckConstructor for FileCopied {
    type Output = Self;

    fn from_check_table(
        generic_check: GenericChecker,
        check_table: toml_edit::Table,
    ) -> Result<Self::Output, CheckDefinitionError> {
        let source = ReadablePath::from_string(
            get_string_value_from_checktable(&check_table, "source")?.as_str(),
            &generic_check.file_with_checks,
        )
        .map_err(|_| CheckDefinitionError::InvalidDefinition("invalid source url".into()))?;

        let destination = if check_table.contains_key("destination") {
            WritablePath::from_string(
                get_string_value_from_checktable(&check_table, "destination")?.as_str(),
            )
            .map_err(|_| {
                CheckDefinitionError::InvalidDefinition("invalid destination path".into())
            })?
        } else {
            let destination_dir = WritablePath::from_string(
                get_string_value_from_checktable(&check_table, "destination_dir")?.as_str(),
            )
            .map_err(|_| {
                CheckDefinitionError::InvalidDefinition("invalid destination_dir path".into())
            })?;

            let file_name = match source.as_ref().path().rsplit_once("/") {
                Some((_, filename)) => filename,
                None => source.as_ref().path(),
            };

            WritablePath::new(destination_dir.as_ref().join(file_name))
        };

        Ok(Self {
            destination,
            source,
            generic_check,
        })
    }
}
impl Checker for FileCopied {
    fn checker_type(&self) -> String {
        "file_copied".to_string()
    }

    fn generic_checker(&self) -> &GenericChecker {
        &self.generic_check
    }
    fn checker_object(&self) -> String {
        self.source.as_ref().to_string()
    }
    fn check_(&self, fix: bool) -> Result<crate::checkers::base::CheckResult, CheckError> {
        // todo check whether the file is changed
        let mut action_messages: Vec<String> = vec![];

        match self.source.exists() {
            Ok(false) => return Err(CheckError::String("source file does not exists".into())),
            Ok(true) => (),
            Err(e) => return Err(CheckError::String(e.to_string())),
        }

        let copy_file_needed = !self.destination.as_ref().exists();

        if copy_file_needed {
            action_messages.push("copy file".into());
        }
        let action_message = action_messages.join("\n");

        let check_result = match (copy_file_needed, fix) {
            (false, _) => CheckResult::NoFixNeeded,
            (true, false) => CheckResult::FixNeeded(action_message),
            (true, true) => {
                if let Some(parent) = self.destination.as_ref().parent() {
                    std::fs::create_dir_all(parent)?;
                }

                match self.source.copy(&self.destination) {
                    Ok(_) => CheckResult::FixExecuted(action_message),
                    Err(e) => return Err(CheckError::String(e.to_string())),
                }
            }
        };

        Ok(check_result)
    }
}

#[cfg(test)]
mod tests {

    use std::fs::write;

    use crate::checkers::{base::CheckResult, test_helpers};

    use super::*;

    use tempfile::tempdir;

    fn get_file_copied_check_with_result(
        source: impl Into<String>,
    ) -> (Result<FileCopied, CheckDefinitionError>, tempfile::TempDir) {
        let generic_check = test_helpers::get_generic_check();

        let mut check_table = toml_edit::Table::new();
        let dir = tempdir().unwrap();
        let destination = dir.path().join("file_to_check");
        check_table.insert(
            "destination",
            destination.to_string_lossy().to_string().into(),
        );
        check_table.insert("source", source.into().into());
        (
            FileCopied::from_check_table(generic_check, check_table),
            dir,
        )
    }

    #[test]
    fn test_file_copied_from_https() {
        let (file_copied_check, _tempdir) = get_file_copied_check_with_result(
            "https://rust-lang.org/static/images/rust-logo-blk.svg",
        );
        let file_copied_check = file_copied_check.expect("no errors");

        assert_eq!(
            file_copied_check.check_(false).unwrap(),
            CheckResult::FixNeeded("copy file".into())
        );

        assert_eq!(
            file_copied_check.check_(true).unwrap(),
            CheckResult::FixExecuted("copy file".into())
        );
        assert_eq!(
            file_copied_check.check_(false).unwrap(),
            CheckResult::NoFixNeeded
        );
    }

    #[test]
    fn test_file_copied_from_fs() {
        let dir = tempdir().unwrap();
        let file_to_copy = dir.path().join("file_to_copy");
        let _ = write(&file_to_copy, "bla");

        let (file_copied_check, _tempdir) =
            get_file_copied_check_with_result(file_to_copy.to_string_lossy().to_string());
        let file_copied_check = file_copied_check.expect("no errors");

        assert_eq!(
            file_copied_check.check_(false).unwrap(),
            CheckResult::FixNeeded("copy file".into())
        );

        assert_eq!(
            file_copied_check.check_(true).unwrap(),
            CheckResult::FixExecuted("copy file".into())
        );
        assert_eq!(
            file_copied_check.check_(false).unwrap(),
            CheckResult::NoFixNeeded
        );

        assert_eq!(
            std::fs::read_to_string(file_copied_check.destination.as_ref()).unwrap(),
            "bla"
        )
    }
}
