use std::path::{Path, PathBuf};

use crate::{Check, Entry, FilePresent};

impl Entry for IniEntryPresent {
    fn new(path: PathBuf, table: toml_edit::Table) -> Self {
        Self { path, table }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn table(&self) -> &toml_edit::Table {
        &self.table
    }
}

impl Check for IniEntryPresent {
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
pub(crate) struct IniEntryAbsent {
    path: PathBuf,
    table: toml_edit::Table,
}
impl Check for IniEntryAbsent {
    fn check(&self) -> Result<(), String> {
        Ok(())
    }

    fn fix(&self) -> Result<(), String> {
        Ok(())
    }
}
