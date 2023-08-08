use super::base::{Action, Check};
use std::{fs, path::PathBuf};

#[derive(Debug)]
pub(crate) struct EntryRegexMatch {
    // path to the file where the checkers are defined
    file_with_checks: PathBuf,
    // path to the file which needs to be checked
    file_to_check: PathBuf,
    value: toml::Table,
}

impl EntryRegexMatch {
    pub fn new(file_with_checks: PathBuf, file_to_check: PathBuf, value: toml::Table) -> Self {
        Self {
            file_with_checks,
            file_to_check,
            value,
        }
    }
}

impl Check for EntryRegexMatch {
    fn check_type(&self) -> String {
        "entry_regex_match".to_string()
    }

    fn file_with_checks(&self) -> &PathBuf {
        &self.file_with_checks
    }

    fn file_to_check(&self) -> &PathBuf {
        &self.file_to_check
    }

    fn check(&self) -> Option<Action> {
        let contents = if !self.file_to_check().exists() {
            "".to_string()
        } else {
            let contents = fs::read_to_string(self.file_to_check());
            if let Err(e) = contents {
                log::error!(
                    "Error: {} {} {} {}",
                    e,
                    self.file_with_checks().to_string_lossy(),
                    self.file_to_check().to_string_lossy(),
                    self.check_type(),
                );
                return None;
            }
            contents.unwrap()
        };

        // Todo: multple actions?
        let action = match self.file_type().validate_regex(&contents, &self.value) {
            Ok(_) => {
                self.print_ok();
                Action::None
            }
            Err(e) => {
                self.print_nok("manual fix regex", e.to_string().as_str());
                Action::Manual(e)
            }
        };
        Some(action)
    }
}
