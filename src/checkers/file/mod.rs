use std::fs;
#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use similar::{DiffableStr, TextDiff};

use crate::{
    checkers::{
        GenericChecker,
        base::{CheckDefinitionError, CheckError, CheckResult, Checker},
    },
    file_types::{self, FileType},
    mapping::generic::Mapping,
    uri::WritablePath,
};

pub(crate) mod entry_absent;
pub(crate) mod entry_present;
pub(crate) mod file_absent;
pub(crate) mod file_copied;
pub(crate) mod file_present;
pub(crate) mod file_unpacked;
pub(crate) mod key_absent;
pub(crate) mod key_value_present;
pub(crate) mod key_value_regex_match;
pub(crate) mod lines_absent;
pub(crate) mod lines_present;

#[derive(Debug, Clone)]
pub(crate) struct FileCheck {
    generic_check: GenericChecker,
    pub(crate) file_to_check: WritablePath,
    pub(crate) file_type_override: Option<String>,
}

impl FileCheck {
    fn from_check_table(
        generic_check: GenericChecker,
        config_table: &toml_edit::Table,
    ) -> Result<Self, CheckDefinitionError> {
        let file_to_check = match config_table.get("file") {
            None => Err(CheckDefinitionError::InvalidDefinition(
                "file is not defined".into(),
            )),
            Some(file_to_check) => match file_to_check.as_str() {
                None => Err(CheckDefinitionError::InvalidDefinition(
                    "file is not a string".into(),
                ))?,
                Some(file_to_check) => Ok(WritablePath::from_string(file_to_check)
                    .map_err(|_| CheckDefinitionError::InvalidDefinition("invalid path".into()))?),
            },
        }?;

        let file_type_override = match config_table.get("file_type") {
            None => Ok(None),
            Some(file_type) => match file_type.as_str() {
                None => Err(CheckDefinitionError::InvalidDefinition(
                    "file_type is not a string".into(),
                )),
                Some(file_type) => Ok(Some(file_type.to_string())),
            },
        }?;

        Ok(Self {
            file_to_check,
            file_type_override,
            generic_check,
        })
    }

    fn check_object(&self) -> String {
        self.file_to_check.as_ref().to_string_lossy().to_string()
    }

    fn file_to_check(&self) -> &PathBuf {
        self.file_to_check.as_ref()
    }

    fn get_action_message(&self, old_contents: &str, new_contents: &str) -> String {
        format!(
            "Set file contents to: \n{}",
            TextDiff::from_lines(
                old_contents,
                // self.generic_check()
                //     .get_file_contents(DefaultContent::EmptyString)
                //     .unwrap_or("".to_string())
                //     .as_str(),
                new_contents
            )
            .unified_diff()
        )
    }

    fn conclude_check_file_exists(
        &self,
        check: &impl Checker,
        placeholder: Option<String>,
        permissions: Option<std::fs::Permissions>,
        regex: Option<regex::Regex>,
        fix: bool,
    ) -> Result<CheckResult, CheckError> {
        let mut action_messages: Vec<String> = vec![];

        let create_file = !self.file_to_check.as_ref().exists();

        if create_file {
            action_messages.push("create file".into());
        }
        let fix_placeholder = if let Some(placeholder) = placeholder.clone() {
            action_messages.push(format!("set contents to {}", placeholder.clone()));
            create_file
        } else {
            false
        };

        let fix_permissions = if let Some(permissions) = permissions.clone() {
            #[cfg(target_os = "windows")]
            {
                false
            }

            #[cfg(not(target_os = "windows"))]
            {
                if create_file {
                    true
                } else {
                    let current_permissions = match self.file_to_check.as_ref().metadata() {
                        Err(_) => {
                            return Err(CheckError::PermissionsNotAccessable);
                        }
                        Ok(metadata) => metadata.permissions(),
                    };

                    // we only check for the last 3 octal digits

                    (current_permissions.mode() & 0o777) != (permissions.mode() & 0o777)
                }
            }
        } else {
            false
        };

        #[cfg(not(target_os = "windows"))]
        if fix_permissions {
            action_messages.push(format!(
                "fix permissions to {:o}",
                permissions.clone().unwrap().to_owned().mode()
            ));
        }

        let fix_regex = if let Some(regex) = regex.clone() {
            create_file || !regex.is_match(self.get_file_contents()?.as_str())
        } else {
            false
        };

        if fix_regex {
            action_messages.push(format!(
                "fix content to match regex {:?}",
                regex.unwrap().to_string()
            ));
        }

        let action_message = action_messages.join("\n");

        let check_result = match (
            create_file || fix_permissions || fix_placeholder || fix_regex,
            fix,
        ) {
            (false, _) => CheckResult::NoFixNeeded,
            (true, false) => CheckResult::FixNeeded(action_message),
            (true, true) => {
                let contents = if fix_placeholder {
                    placeholder.unwrap()
                } else {
                    "".to_string()
                };
                self.set_file_contents(contents)?;
                if fix_permissions {
                    fs::set_permissions(self.file_to_check(), permissions.unwrap())?;
                }
                CheckResult::FixExecuted(action_message)
            }
        };

        Ok(check_result)
    }

    fn conclude_check_new_contents(
        &self,
        check: &impl Checker,
        new_contents: String,
        fix: bool,
    ) -> Result<CheckResult, CheckError> {
        let old_contents = self.get_file_contents()?;

        let action_message = if old_contents == new_contents {
            "".to_string()
        } else {
            self.get_action_message(old_contents.as_str(), new_contents.as_str())
        };

        let check_result = match (old_contents == new_contents, fix) {
            (true, _) => CheckResult::NoFixNeeded,
            (false, false) => CheckResult::FixNeeded(action_message),
            (false, true) => {
                self.set_file_contents(new_contents)?;
                CheckResult::FixExecuted(action_message)
            }
        };

        Ok(check_result)
    }

    fn conclude_check_with_new_doc(
        &self,
        check: &impl Checker,
        new_doc: Box<dyn Mapping>,
        fix: bool,
    ) -> Result<CheckResult, CheckError> {
        self.conclude_check_new_contents(check, new_doc.to_string()?, fix)
    }

    fn conclude_check_with_remove(
        &self,
        check: &impl Checker,
        fix: bool,
    ) -> Result<CheckResult, CheckError> {
        let action_message = "remove file".to_string();

        let check_result = match (self.file_to_check.as_ref().exists(), fix) {
            (false, _) => CheckResult::NoFixNeeded,
            (true, false) => CheckResult::FixNeeded(action_message),
            (true, true) => {
                self.remove_file()?;
                CheckResult::FixExecuted(action_message)
            }
        };

        Ok(check_result)
    }

    fn get_file_contents(&self) -> Result<String, CheckError> {
        match fs::read_to_string(self.file_to_check()) {
            Ok(contents) => {
                let contents = if contents.ends_with_newline() {
                    contents
                } else {
                    format!("{contents}\n")
                };
                Ok(contents)
            }
            Err(_) => Ok("".to_string()),
        }
    }

    fn set_file_contents(&self, contents: String) -> Result<(), CheckError> {
        if fs::exists(self.file_to_check()).expect("no error checking existance of path")
            && contents.is_empty()
        {
            return Ok(());
        }

        if let Some(parent) = self.file_to_check().parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent)?;
        }

        if let Err(e) = fs::write(self.file_to_check(), contents) {
            log::error!(
                "⚠  Cannot write file {} {}",
                self.file_to_check().to_string_lossy(),
                e
            );
            Err(CheckError::FileCanNotBeWritten)
        } else {
            Ok(())
        }
    }

    fn remove_file(&self) -> Result<(), CheckError> {
        if let Err(e) = fs::remove_file(self.file_to_check()) {
            log::error!(
                "⚠ Cannot remove file {} {}",
                self.file_to_check().to_string_lossy(),
                e
            );
            Err(CheckError::FileCanNotBeRemoved)
        } else {
            Ok(())
        }
    }

    fn get_mapping(&self) -> Result<Box<dyn Mapping>, CheckError> {
        let extension = self.file_to_check().extension();
        if extension.is_none() && self.file_type_override.is_none() {
            return Err(CheckError::UnknownFileType(
                "No extension found".to_string(),
            ));
        };

        let contents = self.get_file_contents()?;

        let extension = self.file_type_override.clone().unwrap_or(
            extension
                .expect("file has an extension")
                .to_str()
                .expect("extension is a string")
                .to_string(),
        );

        if extension == "toml" {
            return file_types::toml::Toml::new().to_mapping(&contents);
        } else if extension == "json" {
            return file_types::json::Json::new().to_mapping(&contents);
        } else if extension == "yaml" || extension == "yml" {
            return file_types::yaml::Yaml::new().to_mapping(&contents);
        }
        Err(CheckError::UnknownFileType(extension))
    }
}

pub(crate) fn get_option_string_value_from_checktable(
    check_table: &toml_edit::Table,
    key: &str,
) -> Result<Option<String>, CheckDefinitionError> {
    match check_table.get(key) {
        None => Ok(None),
        Some(value) => match value.as_str() {
            None => Err(CheckDefinitionError::InvalidDefinition(format!(
                "{key} is not a string"
            ))),
            Some(value) => Ok(Some(value.to_string())),
        },
    }
}

pub(crate) fn get_string_value_from_checktable(
    check_table: &toml_edit::Table,
    key: &str,
) -> Result<String, CheckDefinitionError> {
    match get_option_string_value_from_checktable(check_table, key) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(CheckDefinitionError::InvalidDefinition(format!(
            "{key} is not present in check_table"
        ))),
        Err(err) => Err(err),
    }
}
