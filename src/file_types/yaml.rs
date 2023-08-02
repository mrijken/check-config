use std::path::PathBuf;

use crate::{Check, FilePresent};

#[derive(Debug)]
pub struct YamlEntryPresent {
    path: PathBuf,
    table: toml::Table,
}

impl Check for YamlEntryPresent {
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
pub struct YamlEntryAbsent {
    path: PathBuf,
    table: toml::Table,
}
impl Check for YamlEntryAbsent {
    fn check(&self) -> Result<(), String> {
        Ok(())
    }

    fn fix(&self) -> Result<(), String> {
        Ok(())
    }
}
