use super::base::{Action, Check, IstAndSoll};
use std::{fs, path::PathBuf};

#[derive(Debug)]
pub(crate) struct LinesPresent {
    // path to the file where the checkers are defined
    file_with_checks: PathBuf,
    // path to the file which needs to be checked
    file_to_check: PathBuf,
    lines: String,
}

impl LinesPresent {
    pub fn new(file_with_checks: PathBuf, file_to_check: PathBuf, lines: String) -> Self {
        Self {
            file_with_checks,
            file_to_check,
            lines,
        }
    }
}

impl Check for LinesPresent {
    fn check_type(&self) -> String {
        "lines_present".to_string()
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
                self.lines.clone(),
                Action::SetContents,
            ));
        }
        match fs::read_to_string(self.file_to_check()) {
            Ok(contents) => {
                if contents.contains(&self.lines) {
                    Ok(IstAndSoll::new(
                        contents.clone(),
                        contents.clone(),
                        Action::None,
                    ))
                } else {
                    let mut new_contents = contents.clone();
                    if !new_contents.ends_with('\n') {
                        new_contents += "\n";
                    }
                    new_contents += &self.lines.clone();
                    Ok(IstAndSoll::new(
                        contents.clone(),
                        new_contents,
                        Action::SetContents,
                    ))
                }
            }
            Err(err) => Err(err.to_string()),
        }
    }
}
