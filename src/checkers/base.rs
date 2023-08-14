use similar::TextDiff;

use core::{fmt::Debug as DebugTrait, panic};
use std::io;
use thiserror::Error;

use super::GenericCheck;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Action {
    RemoveFile,
    SetContents(String),
    Manual(String),
    None,
}

#[derive(Error, Debug)]
pub enum CheckError {
    #[error("file can not be read")]
    FileCanNotBeRead(#[from] io::Error),
    #[error("unknown file type; do not know how to handle")]
    UnknownFileType(String),
    #[error("file can not be removed")]
    FileCanNotBeRemoved,
    #[error("file can not be written")]
    FileCanNotBeWritten,
    #[error("invalid regex")]
    InvalidRegex(String),
    #[error("key not found")]
    KeyNotFound(String),
    #[error("invalid file format")]
    InvalidFileFormat(String),
}

pub(crate) trait Check: DebugTrait {
    fn check_type(&self) -> String;
    fn generic_check(&self) -> &GenericCheck;
    fn get_action(&self) -> Result<Action, CheckError> {
        panic!("Function is not implemented");
    }

    fn print_ok(&self) {
        log::info!(
            "✅ {} - {} - {}",
            self.generic_check().file_with_checks().to_string_lossy(),
            self.generic_check().file_to_check().to_string_lossy(),
            self.check_type(),
        );
    }
    fn print_nok(&self, message_type: &str, message: &str) {
        log::info!(
            "❌ {} - {} - {} - {}\n{}",
            self.generic_check().file_with_checks().to_string_lossy(),
            self.generic_check().file_to_check().to_string_lossy(),
            self.check_type(),
            message_type,
            message,
        );
    }
    fn check(&self) -> Result<Action, CheckError> {
        let action = match self.get_action() {
            Ok(ist_and_soll) => ist_and_soll,
            Err(e) => {
                log::error!(
                    "Error: {} {} {} {}",
                    e,
                    self.generic_check().file_with_checks().to_string_lossy(),
                    self.generic_check().file_to_check().to_string_lossy(),
                    self.check_type(),
                );
                return Err(e);
            }
        };
        match action.clone() {
            Action::None => {
                self.print_ok();
            }
            Action::RemoveFile => {
                self.print_nok("file is present", "remove file");
            }
            Action::SetContents(new_contents) => {
                self.print_nok(
                    "file contents are different",
                    &format!(
                        "{}",
                        TextDiff::from_lines(
                            self.generic_check()
                                .get_file_contents()
                                .unwrap_or("".to_string())
                                .as_str(),
                            new_contents.as_str()
                        )
                        .unified_diff()
                    ),
                );
            }
            Action::Manual(action) => {
                self.print_nok("manual action", action.clone().as_str());
            }
        }

        Ok(action)
    }

    fn fix(&self) -> Result<Action, CheckError> {
        log::info!(
            "Fixing file {}",
            self.generic_check().file_to_check().to_string_lossy()
        );
        let action = self.check()?;
        match action {
            Action::RemoveFile => {
                self.generic_check().remove_file()?;
                Ok(Action::None)
            }
            Action::SetContents(new_contents) => {
                self.generic_check().set_file_contents(new_contents)?;
                Ok(Action::None)
            }
            Action::Manual(m) => Ok(Action::Manual(m)),
            Action::None => Ok(Action::None),
        }
    }
}
