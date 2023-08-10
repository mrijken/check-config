use std::path::PathBuf;

use super::base::{Action, Check, CheckError};

#[derive(Debug)]
pub(crate) struct FilePresent {
    // path to the file where the checkers are defined
    file_with_checks: PathBuf,
    // path to the file which needs to be checked
    file_to_check: PathBuf,
}

impl FilePresent {
    pub fn new(file_with_checks: PathBuf, file_to_check: PathBuf) -> Self {
        Self {
            file_with_checks,
            file_to_check,
        }
    }
}

impl Check for FilePresent {
    fn check_type(&self) -> String {
        "file_present".to_string()
    }

    fn file_with_checks(&self) -> &PathBuf {
        &self.file_with_checks
    }

    fn file_to_check(&self) -> &PathBuf {
        &self.file_to_check
    }

    fn get_action(&self) -> Result<Action, CheckError> {
        match self.file_to_check.exists() {
            false => Ok(Action::SetContents("".to_string())),
            true => Ok(Action::None),
        }
    }
}
