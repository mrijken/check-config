use super::base::{Action, Check, IstAndSoll};
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

    fn get_ist_and_soll(&self) -> Result<IstAndSoll, String> {
        if !self.file_to_check().exists() {
            return Ok(IstAndSoll::new(
                "".to_string(),
                "".to_string(),
                Action::RemoveFile,
            ));
        }

        match fs::read_to_string(self.file_to_check()) {
            Ok(contents) => {
                if contents.contains(&self.lines) {
                    let new_contents = contents.replace(&self.lines, "");
                    Ok(IstAndSoll::new(contents, new_contents, Action::SetContents))
                } else {
                    Ok(IstAndSoll::new(contents.clone(), contents, Action::None))
                }
            }
            Err(_) => Err("error performing check".to_string()),
        }
    }
}
