use super::base::{Action, Check, CheckError};
use super::GenericCheck;

#[derive(Debug)]
pub(crate) struct FileAbsent {
    generic_check: GenericCheck,
}

impl FileAbsent {
    pub fn new(generic_check: GenericCheck) -> Self {
        Self { generic_check }
    }
}

impl Check for FileAbsent {
    fn check_type(&self) -> String {
        "file_absent".to_string()
    }

    fn generic_check(&self) -> &GenericCheck {
        &self.generic_check
    }

    fn get_action(&self) -> Result<Action, CheckError> {
        match self.generic_check().file_to_check().exists() {
            true => Ok(Action::RemoveFile),
            false => Ok(Action::None),
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
        let file_with_checks = crate::uri::Uri::Path(dir.path().join("file_with_checks"));
        let generic_check = GenericCheck {
            file_to_check,
            file_type_override: None,
            file_with_checks,
        };

        let file_absent_check = FileAbsent::new(generic_check);

        assert_eq!(file_absent_check.check().unwrap(), Action::None);

        File::create(file_absent_check.generic_check().file_to_check()).unwrap();
        assert_eq!(file_absent_check.check().unwrap(), Action::RemoveFile);
    }
}
