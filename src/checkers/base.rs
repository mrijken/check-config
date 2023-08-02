use crate::file_types::{self, FileType};
use core::{fmt::Debug as DebugTrait, panic};
use std::{ffi::OsStr, fs, path::PathBuf};

pub(crate) enum Action {
    RemoveFile,
    SetContents,
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
    fn get_ist_and_soll(&self) -> Result<IstAndSoll, String>;
    fn checkers_path(&self) -> &PathBuf;
    fn config_path(&self) -> &PathBuf;

    fn check(&self) -> Result<IstAndSoll, String> {
        self.get_ist_and_soll()
    }

    fn fix(&self) -> Result<(), String> {
        log::info!("Fixing file {}", self.config_path().to_string_lossy());
        let ist_and_soll = self.get_ist_and_soll()?;
        match ist_and_soll.action {
            Action::RemoveFile => match fs::remove_file(self.config_path()) {
                Ok(()) => Ok(()),
                Err(e) => {
                    log::error!(
                        "Cannot remove file {} {}",
                        self.config_path().to_string_lossy(),
                        e
                    );
                    Err("Cannot remove file".to_string())
                }
            },
            Action::SetContents => match fs::write(self.config_path(), ist_and_soll.soll) {
                Ok(()) => Ok(()),
                Err(e) => {
                    log::error!(
                        "Cannot write file {} {}",
                        self.config_path().to_string_lossy(),
                        e
                    );
                    Err("Cannot write file".to_string())
                }
            },
            Action::None => Ok(()),
        }
    }

    /// Get the file type of the config_path
    fn file_type(&self) -> Box<dyn FileType> {
        if self.config_path().extension() == Some(OsStr::new("toml")) {
            return Box::new(file_types::toml::Toml::new());
            // } else if self.config_path().extension() == Some(OsStr::new("json")) {
            //     return file_types::json::Json;
            // } else if self.config_path().extension() == Some(OsStr::new("yaml"))
            //     || self.config_path().extension() == Some(OsStr::new("yml"))
            // {
            //     return FileType::Yaml;
            // } else if self.config_path().extension() == Some(OsStr::new("ini")) {
            //     return FileType::Ini;
        }
        panic!("Unknown file type");
    }
}
