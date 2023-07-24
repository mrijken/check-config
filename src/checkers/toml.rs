use std::path::PathBuf;

use crate::{Check, FilePresent};

#[derive(Debug)]
pub struct TomlEntryPresent {
    path: PathBuf,
    table: toml::Table,
}

impl Check for TomlEntryPresent {
    fn check(&self) -> Result<(), String> {
        FilePresent {
            path: self.path.clone(),
        }
        .check()?;
        Ok(())
    }

    fn fix(&self) -> Result<(), String> {
        FilePresent {
            path: self.path.clone(),
        }
        .fix()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct TomlEntryAbsent {
    path: PathBuf,
    table: toml::Table,
}
impl Check for TomlEntryAbsent {
    fn check(&self) -> Result<(), String> {
        Ok(())
    }

    fn fix(&self) -> Result<(), String> {
        Ok(())
    }
}
