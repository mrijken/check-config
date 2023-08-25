use similar::TextDiff;

use core::{fmt::Debug as DebugTrait, panic};
use std::io;
use thiserror::Error;

use super::GenericCheck;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Action {
    RemoveFile,
    SetContents(String),
    MatchRegex { key: String, regex: String },
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
    #[error("invalid file format")]
    InvalidFileFormat(String),
    #[error("invalid regex format")]
    InvalidRegex(String),
}

pub(crate) trait Check: DebugTrait {
    fn check_type(&self) -> String;
    fn generic_check(&self) -> &GenericCheck;
    fn get_action(&self) -> Result<Action, CheckError> {
        panic!("Function is not implemented");
    }

    fn print(&self, is_ok: bool, key: Option<&str>, action_message: Option<&str>) {
        let key = match key {
            Some(k) => format!(" - {}", k),
            None => "".to_string(),
        };
        let ok = match is_ok {
            true => "✅",
            false => "❌",
        };
        let action_message = match action_message {
            Some(msg) => format!(" - {}", msg),
            None => "".to_string(),
        };
        let msg = format!(
            "{} {} - {} - {}{}{}",
            ok,
            self.generic_check().file_with_checks().to_string_lossy(),
            self.generic_check().file_to_check().to_string_lossy(),
            self.check_type(),
            key,
            action_message
        );
        match is_ok {
            true => log::info!("{}", msg),
            false => log::error!("{}", msg),
        }
    }
    fn check(&self) -> Result<Action, CheckError> {
        let action = match self.get_action() {
            Ok(ist_and_soll) => ist_and_soll,
            Err(e) => {
                self.print(false, None, Some(&e.to_string()));
                return Err(e);
            }
        };
        match action.clone() {
            Action::None => {
                self.print(true, None, None);
            }
            Action::RemoveFile => {
                self.print(false, None, Some("remove file"));
            }
            Action::SetContents(new_contents) => {
                self.print(
                    false,
                    None,
                    Some(&format!(
                        "Set file contents to: {}",
                        TextDiff::from_lines(
                            self.generic_check()
                                .get_file_contents(Some("".to_string()))
                                .unwrap_or("".to_string())
                                .as_str(),
                            new_contents.as_str()
                        )
                        .unified_diff()
                    )),
                );
            }
            Action::MatchRegex { key, regex } => {
                self.print(
                    false,
                    Some(&key),
                    Some(&format!("Make sure value matches regex {}", regex)),
                );
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
            action => Ok(action),
        }
    }
}
