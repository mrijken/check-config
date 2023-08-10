use similar::TextDiff;

use crate::file_types::{self, FileType};
use core::{fmt::Debug as DebugTrait, panic};
use std::{ffi::OsStr, fs, io, path::PathBuf};
use thiserror::Error;

#[derive(Clone, Debug)]
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
}

pub(crate) trait Check: DebugTrait {
    fn check_type(&self) -> String;
    fn get_action(&self) -> Result<Action, CheckError> {
        panic!("Function is not implemented");
    }
    fn file_with_checks(&self) -> &PathBuf;
    fn file_to_check(&self) -> &PathBuf;

    fn get_file_contents(&self) -> io::Result<String> {
        fs::read_to_string(self.file_to_check())
    }

    fn print_ok(&self) {
        log::info!(
            "✅ {} - {} - {}",
            self.file_with_checks().to_string_lossy(),
            self.file_to_check().to_string_lossy(),
            self.check_type(),
        );
    }
    fn print_nok(&self, message_type: &str, message: &str) {
        log::info!(
            "❌ {} - {} - {} - {}\n{}",
            self.file_with_checks().to_string_lossy(),
            self.file_to_check().to_string_lossy(),
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
                    self.file_with_checks().to_string_lossy(),
                    self.file_to_check().to_string_lossy(),
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
                            self.get_file_contents().unwrap().as_str(),
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

    fn fix(&self) -> Result<(), CheckError> {
        log::info!("Fixing file {}", self.file_to_check().to_string_lossy());
        let action = self.check()?;
        match action {
            Action::RemoveFile => match fs::remove_file(self.file_to_check()) {
                Ok(()) => Ok(()),
                Err(e) => {
                    log::error!(
                        "Cannot remove file {} {}",
                        self.file_to_check().to_string_lossy(),
                        e
                    );
                    Err(CheckError::FileCanNotBeRemoved)
                }
            },
            Action::SetContents(new_contents) => {
                match fs::write(self.file_to_check(), new_contents) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        log::error!(
                            "Cannot write file {} {}",
                            self.file_to_check().to_string_lossy(),
                            e
                        );
                        Err(CheckError::FileCanNotBeWritten)
                    }
                }
            }
            Action::Manual(_) => Ok(()),
            Action::None => Ok(()),
        }
    }

    /// Get the file type of the file_to_check
    fn file_type(&self) -> Result<Box<dyn FileType>, CheckError> {
        let extension = self.file_to_check().extension();
        if extension.is_none() {
            return Err(CheckError::UnknownFileType(
                "No extension found".to_string(),
            ));
        };

        let extension = extension.unwrap();

        if extension == OsStr::new("toml") {
            return Ok(Box::new(file_types::toml::Toml::new()));
            // } else if extension == Some(OsStr::new("json")) {
            //     return file_types::json::Json;
            // } else if extension == Some(OsStr::new("yaml"))
            //     || extension == Some(OsStr::new("yml"))
            // {
            //     return FileType::Yaml;
            // } else if extension == Some(OsStr::new("ini")) {
            //     return FileType::Ini;
        }
        Err(CheckError::UnknownFileType(
            extension.to_string_lossy().to_string(),
        ))
    }
}
