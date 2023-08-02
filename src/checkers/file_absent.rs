use std::path::PathBuf;

use super::base::{Action, Check, IstAndSoll};

#[derive(Debug)]
pub(crate) struct FileAbsent {
    // path to the file where the checkers are defined
    checkers_path: PathBuf,
    // path to the file which needs to be checked
    config_path: PathBuf,
}

impl FileAbsent {
    pub fn new(checkers_path: PathBuf, config_path: PathBuf) -> Self {
        Self {
            checkers_path,
            config_path,
        }
    }
}

impl Check for FileAbsent {
    fn check_type(&self) -> String {
        "file_absent".to_string()
    }

    fn checkers_path(&self) -> &PathBuf {
        &self.checkers_path
    }

    fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    fn get_ist_and_soll(&self) -> Result<IstAndSoll, String> {
        match self.config_path.exists() {
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
