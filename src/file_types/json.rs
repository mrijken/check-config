use std::path::PathBuf;

use crate::{Check, FilePresent};

#[derive(Debug)]
pub struct JsonEntryPresent {
    path: PathBuf,
    table: toml::Table,
}

impl Check for JsonEntryPresent {
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
pub struct JsonEntryAbsent {
    path: PathBuf,
    table: toml::Table,
}
impl Check for JsonEntryAbsent {
    fn check(&self) -> Result<(), String> {
        Ok(())
    }

    fn fix(&self) -> Result<(), String> {
        Ok(())
    }
}
