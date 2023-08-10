use super::base::{Action, Check, CheckError};
use std::{fs, path::PathBuf};

#[derive(Debug)]
pub(crate) struct LinesAbsent {
    // path to the file where the checkers are defined
    file_with_checks: PathBuf,
    // path to the file which needs to be checked
    file_to_check: PathBuf,
    lines: String,
}

impl LinesAbsent {
    pub fn new(file_with_checks: PathBuf, file_to_check: PathBuf, lines: String) -> Self {
        Self {
            file_with_checks,
            file_to_check,
            lines,
        }
    }
}

impl Check for LinesAbsent {
    fn check_type(&self) -> String {
        "lines_absent".to_string()
    }

    fn file_with_checks(&self) -> &PathBuf {
        &self.file_with_checks
    }

    fn file_to_check(&self) -> &PathBuf {
        &self.file_to_check
    }

    fn get_action(&self) -> Result<Action, CheckError> {
        if !self.file_to_check().exists() {
            return Ok(Action::RemoveFile);
        }

        let contents =
            fs::read_to_string(self.file_to_check()).map_err(CheckError::FileCanNotBeRead)?;
        if contents.contains(&self.lines) {
            let new_contents = contents.replace(&self.lines, "");
            Ok(Action::SetContents(new_contents))
        } else {
            Ok(Action::None)
        }
    }
}
