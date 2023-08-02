use super::base::{Action, Check, IstAndSoll};
use std::{fs, path::PathBuf};

#[derive(Debug)]
pub(crate) struct LinesAbsent {
    // path to the file where the checkers are defined
    checkers_path: PathBuf,
    // path to the file which needs to be checked
    config_path: PathBuf,
    lines: String,
}

impl LinesAbsent {
    pub fn new(checkers_path: PathBuf, config_path: PathBuf, lines: String) -> Self {
        Self {
            checkers_path,
            config_path,
            lines,
        }
    }
}

impl Check for LinesAbsent {
    fn check_type(&self) -> String {
        "lines_absent".to_string()
    }

    fn checkers_path(&self) -> &PathBuf {
        &self.checkers_path
    }

    fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    fn get_ist_and_soll(&self) -> Result<IstAndSoll, String> {
        if !self.config_path().exists() {
            return Ok(IstAndSoll::new(
                "".to_string(),
                "".to_string(),
                Action::RemoveFile,
            ));
        }

        match fs::read_to_string(self.config_path()) {
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
