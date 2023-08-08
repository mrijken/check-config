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
pub enum ActionError {
    #[error("file not found")]
    FileNotFound,
}

pub(crate) trait Check: DebugTrait {
    fn check_type(&self) -> String;
    fn get_action(&self) -> Result<Action, String> {
        Err("Function is not implemented".to_string())
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
    fn check(&self) -> Option<Action> {
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
                return None;
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

        Some(action)
    }

    fn fix(&self) {
        log::info!("Fixing file {}", self.file_to_check().to_string_lossy());
        let action = match self.check() {
            Some(action) => action,
            None => {
                log::error!("Due to check error, fix can not be done");
                return;
            }
        };
        match action {
            Action::RemoveFile => match fs::remove_file(self.file_to_check()) {
                Ok(()) => (),
                Err(e) => {
                    log::error!(
                        "Cannot remove file {} {}",
                        self.file_to_check().to_string_lossy(),
                        e
                    );
                }
            },
            Action::SetContents(new_contents) => {
                match fs::write(self.file_to_check(), new_contents) {
                    Ok(()) => (),
                    Err(e) => {
                        log::error!(
                            "Cannot write file {} {}",
                            self.file_to_check().to_string_lossy(),
                            e
                        );
                    }
                }
            }
            Action::Manual(_) => (),
            Action::None => (),
        }
    }

    /// Get the file type of the file_to_check
    fn file_type(&self) -> Box<dyn FileType> {
        if self.file_to_check().extension() == Some(OsStr::new("toml")) {
            return Box::new(file_types::toml::Toml::new());
            // } else if self.file_to_check().extension() == Some(OsStr::new("json")) {
            //     return file_types::json::Json;
            // } else if self.file_to_check().extension() == Some(OsStr::new("yaml"))
            //     || self.file_to_check().extension() == Some(OsStr::new("yml"))
            // {
            //     return FileType::Yaml;
            // } else if self.file_to_check().extension() == Some(OsStr::new("ini")) {
            //     return FileType::Ini;
        }
        panic!("Unknown file type");
    }
}
