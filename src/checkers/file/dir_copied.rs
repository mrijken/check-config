use crate::{
    checkers::{
        base::CheckResult, file::get_string_value_from_checktable,
        get_option_boolean_from_check_table,
    },
    uri::WritablePath,
};

use super::super::{
    GenericChecker,
    base::{CheckConstructor, CheckDefinitionError, CheckError, Checker},
};

#[derive(Debug)]
pub(crate) struct DirCopied {
    source: WritablePath,
    destination: WritablePath,
    generic_check: GenericChecker,
    always_copy_on_fix: bool,
}

//[[dir_copied]]
// source = "path directory to copy"
// destination = "path in which the directory contents will be copied"
// desintation_dir = "path in which the directory will be copied"
//
// check if file is copied
// if source is a relative path, it's relative to the check file, so the dir
// which contain the file which defines this check.
impl CheckConstructor for DirCopied {
    type Output = Self;

    fn from_check_table(
        generic_check: GenericChecker,
        check_table: toml_edit::Table,
    ) -> Result<Self::Output, CheckDefinitionError> {
        let source = WritablePath::from_string(
            get_string_value_from_checktable(&check_table, "source")?.as_str(),
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

            let file_name = match source.as_ref().file_name() {
                Some(filename) => filename,
                None => {
                    return Err(CheckDefinitionError::InvalidDefinition(
                        "Source has to filename".into(),
                    ));
                }
            };

            WritablePath::new(destination_dir.as_ref().join(file_name))
        };

        let always_copy_on_fix =
            get_option_boolean_from_check_table(&check_table, "always_copy_on_fix")?
                .unwrap_or(false);

        Ok(Self {
            destination,
            source,
            generic_check,
            always_copy_on_fix,
        })
    }
}

/// Recursively copy all contents of `src` into `dst`.
fn copy_dir_contents(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    // Create destination directory if it doesn’t exist
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            // Recurse into subdirectory
            copy_dir_contents(&path, &dest_path)?;
        } else {
            // Copy file
            std::fs::copy(&path, &dest_path)?;
        }
    }

    Ok(())
}

impl Checker for DirCopied {
    fn checker_type(&self) -> String {
        "dir_copied".to_string()
    }

    fn generic_checker(&self) -> &GenericChecker {
        &self.generic_check
    }
    fn checker_object(&self) -> String {
        self.source.as_ref().to_string_lossy().to_string()
    }
    fn check_(&self, fix: bool) -> Result<crate::checkers::base::CheckResult, CheckError> {
        // TODO: check whether the file is changed
        let mut action_messages: Vec<String> = vec![];

        if !self.source.exists() {
            return Err(CheckError::String("source dir does not exists".into()));
        }

        // TODO: check also all subdirs and files
        let copy_dir_needed = !self.destination.exists()
            || self.always_copy_on_fix;

        if copy_dir_needed {
            action_messages.push("copy dir".into());
        }
        let action_message = action_messages.join("\n");

        let check_result = match (copy_dir_needed, fix) {
            (false, _) => CheckResult::NoFixNeeded,
            (true, false) => CheckResult::FixNeeded(action_message),
            (true, true) => {
                match copy_dir_contents(self.source.as_ref(), self.destination.as_ref()) {
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

    use crate::checkers::{base::CheckResult, test_helpers};

    use super::*;

    use tempfile::tempdir;

    fn get_dir_copied_check_with_result()
    -> (Result<DirCopied, CheckDefinitionError>, tempfile::TempDir) {
        let generic_check = test_helpers::get_generic_check();

        let mut check_table = toml_edit::Table::new();
        let dir = tempdir().unwrap();
        let source = dir.path().join("source");
        let subdir = source.join("subdir");
        std::fs::create_dir_all(&subdir).unwrap();
        let _ = std::fs::create_dir(&subdir);
        let file = subdir.join("file");
        std::fs::File::create(file).unwrap();
        let destination = dir.path().join("destination");
        check_table.insert(
            "destination",
            destination.to_string_lossy().to_string().into(),
        );
        check_table.insert("source", source.to_string_lossy().to_string().into());
        (DirCopied::from_check_table(generic_check, check_table), dir)
    }

    #[test]
    fn test_dir_copied_from_fs() {
        let (dir_copied_check, _tempdir) = get_dir_copied_check_with_result();
        let dir_copied_check = dir_copied_check.expect("no errors");

        assert_eq!(
            dir_copied_check.check_(false).unwrap(),
            CheckResult::FixNeeded("copy dir".into())
        );

        assert_eq!(
            dir_copied_check.check_(true).unwrap(),
            CheckResult::FixExecuted("copy dir".into())
        );
        assert_eq!(
            dir_copied_check.check_(false).unwrap(),
            CheckResult::NoFixNeeded
        );

        assert!(dir_copied_check.destination.as_ref().exists());
        assert!(
            dir_copied_check
                .destination
                .as_ref()
                .join("subdir/file")
                .exists()
        );
    }
}
