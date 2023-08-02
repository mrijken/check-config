use super::base::{Action, Check, IstAndSoll};
use std::{fs, path::PathBuf};

#[derive(Debug)]
pub(crate) struct EntryAbsent {
    // path to the file where the checkers are defined
    checkers_path: PathBuf,
    // path to the file which needs to be checked
    config_path: PathBuf,
    value: toml::Table,
}

impl EntryAbsent {
    pub fn new(checkers_path: PathBuf, config_path: PathBuf, value: toml::Table) -> Self {
        Self {
            checkers_path,
            config_path,
            value,
        }
    }
}

impl Check for EntryAbsent {
    fn check_type(&self) -> String {
        "entry_absent".to_string()
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
                Action::None,
            ));
        }

        let contents = fs::read_to_string(self.config_path());
        if let Err(s) = contents {
            return Err(s.to_string());
        }
        let contents = contents.unwrap();

        let new_contents = self.file_type().unset(&contents, &self.value).unwrap();

        let action = if contents == new_contents {
            Action::None
        } else {
            Action::SetContents
        };

        Ok(IstAndSoll::new(contents, new_contents, action))
    }
}
