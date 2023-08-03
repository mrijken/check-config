use similar::TextDiff;

use crate::file_types::{self, FileType};
use core::{fmt::Debug as DebugTrait, panic};
use std::{ffi::OsStr, fs, path::PathBuf};

pub(crate) enum Action {
    RemoveFile,
    SetContents,
    Manual(String),
    None,
}
pub(crate) struct IstAndSoll {
    ist: String,
    soll: String,
    action: Action,
}

impl IstAndSoll {
    pub fn new(ist: String, soll: String, action: Action) -> Self {
        Self { ist, soll, action }
    }

    pub fn ist(&self) -> &str {
        &self.ist
    }

    pub fn soll(&self) -> &str {
        &self.soll
    }

    pub fn action(&self) -> &Action {
        &self.action
    }
}

pub(crate) trait Check: DebugTrait {
    fn check_type(&self) -> String;
    fn get_ist_and_soll(&self) -> Result<IstAndSoll, String> {
        Err("Function is not implemented".to_string())
    }
    fn file_with_checks(&self) -> &PathBuf;
    fn file_to_check(&self) -> &PathBuf;

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
    fn check(&self) -> Option<IstAndSoll> {
        let ist_and_soll = match self.get_ist_and_soll() {
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
        match ist_and_soll.action() {
            Action::None => {
                self.print_ok();
            }
            Action::RemoveFile => {
                self.print_nok("file is present", "remove file");
            }
            Action::SetContents => {
                self.print_nok(
                    "file contents are different",
                    &format!(
                        "{}",
                        TextDiff::from_lines(ist_and_soll.ist(), ist_and_soll.soll())
                            .unified_diff()
                    ),
                );
            }
            Action::Manual(action) => {
                self.print_nok("manual action", action);
            }
        }

        Some(ist_and_soll)
    }

    fn fix(&self) {
        log::info!("Fixing file {}", self.file_to_check().to_string_lossy());
        let ist_and_soll = match self.check() {
            Some(ist_and_soll) => ist_and_soll,
            None => {
                log::error!("Due to check error, fix can not be done");
                return;
            }
        };
        match ist_and_soll.action {
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
            Action::SetContents => match fs::write(self.file_to_check(), ist_and_soll.soll) {
                Ok(()) => (),
                Err(e) => {
                    log::error!(
                        "Cannot write file {} {}",
                        self.file_to_check().to_string_lossy(),
                        e
                    );
                }
            },
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
