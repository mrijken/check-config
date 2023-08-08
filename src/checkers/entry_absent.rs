use super::base::{Action, Check};
use std::{fs, path::PathBuf};

#[derive(Debug)]
pub(crate) struct EntryAbsent {
    // path to the file where the checkers are defined
    file_with_checks: PathBuf,
    // path to the file which needs to be checked
    file_to_check: PathBuf,
    value: toml::Table,
}

impl EntryAbsent {
    pub fn new(file_with_checks: PathBuf, file_to_check: PathBuf, value: toml::Table) -> Self {
        Self {
            file_with_checks,
            file_to_check,
            value,
        }
    }
}

impl Check for EntryAbsent {
    fn check_type(&self) -> String {
        "entry_absent".to_string()
    }

    fn file_with_checks(&self) -> &PathBuf {
        &self.file_with_checks
    }

    fn file_to_check(&self) -> &PathBuf {
        &self.file_to_check
    }

    fn get_action(&self) -> Result<Action, String> {
        if !self.file_to_check().exists() {
            return Ok(Action::None);
        }

        let contents = fs::read_to_string(self.file_to_check());
        if let Err(s) = contents {
            return Err(s.to_string());
        }
        let contents = contents.unwrap();

        let new_contents = self.file_type().unset(&contents, &self.value).unwrap();

        let action = if contents == new_contents {
            Action::None
        } else {
            Action::SetContents(new_contents)
        };

        Ok(action)
    }
}
