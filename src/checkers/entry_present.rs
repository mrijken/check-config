use super::base::{Action, Check, IstAndSoll};
use std::{fs, path::PathBuf};

#[derive(Debug)]
pub(crate) struct EntryPresent {
    // path to the file where the checkers are defined
    file_with_checks: PathBuf,
    // path to the file which needs to be checked
    file_to_check: PathBuf,
    value: toml::Table,
}

impl EntryPresent {
    pub fn new(file_with_checks: PathBuf, file_to_check: PathBuf, value: toml::Table) -> Self {
        Self {
            file_with_checks,
            file_to_check,
            value,
        }
    }
}

impl Check for EntryPresent {
    fn check_type(&self) -> String {
        "entry_present".to_string()
    }

    fn file_with_checks(&self) -> &PathBuf {
        &self.file_with_checks
    }

    fn file_to_check(&self) -> &PathBuf {
        &self.file_to_check
    }

    fn get_ist_and_soll(&self) -> Result<IstAndSoll, String> {
        let contents = if !self.file_to_check().exists() {
            "".to_string()
        } else {
            let contents = fs::read_to_string(self.file_to_check());
            if let Err(s) = contents {
                return Err(s.to_string());
            }
            contents.unwrap()
        };

        let new_contents = self.file_type().set(&contents, &self.value).unwrap();

        let action = if contents == new_contents {
            Action::None
        } else {
            Action::SetContents
        };
        Ok(IstAndSoll::new(contents, new_contents, action))
    }
}
