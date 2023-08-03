use std::path::PathBuf;

use super::base::{Action, Check, IstAndSoll};

#[derive(Debug)]
pub(crate) struct FileAbsent {
    // path to the file where the checkers are defined
    file_with_checks: PathBuf,
    // path to the file which needs to be checked
    file_to_check: PathBuf,
}

impl FileAbsent {
    pub fn new(file_with_checks: PathBuf, file_to_check: PathBuf) -> Self {
        Self {
            file_with_checks,
            file_to_check,
        }
    }
}

impl Check for FileAbsent {
    fn check_type(&self) -> String {
        "file_absent".to_string()
    }

    fn file_with_checks(&self) -> &PathBuf {
        &self.file_with_checks
    }

    fn file_to_check(&self) -> &PathBuf {
        &self.file_to_check
    }

    fn get_ist_and_soll(&self) -> Result<IstAndSoll, String> {
        match self.file_to_check.exists() {
            true => Ok(IstAndSoll::new(
                "".to_string(),
                "".to_string(),
                Action::RemoveFile,
            )),

            false => Ok(IstAndSoll::new(
                "".to_string(),
                "".to_string(),
                Action::None,
            )),
        }
    }
}
