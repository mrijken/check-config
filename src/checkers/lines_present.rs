use super::base::{Action, Check, IstAndSoll};
use std::{fs, path::PathBuf};

#[derive(Debug)]
pub(crate) struct LinesPresent {
    // path to the file where the checkers are defined
    checkers_path: PathBuf,
    // path to the file which needs to be checked
    config_path: PathBuf,
    lines: String,
}

impl LinesPresent {
    pub fn new(checkers_path: PathBuf, config_path: PathBuf, lines: String) -> Self {
        Self {
            checkers_path,
            config_path,
            lines,
        }
    }
}

impl Check for LinesPresent {
    fn check_type(&self) -> String {
        "lines_present".to_string()
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
                self.lines.clone(),
                Action::SetContents,
            ));
        }
        match fs::read_to_string(self.config_path()) {
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
