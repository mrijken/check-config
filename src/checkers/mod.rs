use std::{
    env,
    fs::{self},
    path::PathBuf,
    str::FromStr,
};

use base::CheckConstructor;
use similar::DiffableStr;

use crate::{
    file_types::{self, FileType},
    mapping::generic::Mapping,
    uri::{self, expand_to_absolute},
};

use self::base::{Check, CheckDefinitionError, CheckError};

pub(crate) mod base;
pub(crate) mod entry_absent;
pub(crate) mod entry_present;
pub(crate) mod file_absent;
pub(crate) mod file_present;
pub(crate) mod file_regex;
pub(crate) mod key_absent;
pub(crate) mod key_value_present;
pub(crate) mod key_value_regex_match;
pub(crate) mod lines_absent;
pub(crate) mod lines_present;
pub(crate) mod test_helpers;
pub(crate) mod utils;

pub(crate) trait RelativeUrl {
    fn short_url_str(&self) -> String;
}

impl RelativeUrl for url::Url {
    fn short_url_str(&self) -> String {
        let cwd_url = url::Url::parse(&format!(
            "file://{}",
            env::current_dir()
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap()
        ))
        .unwrap();
        match cwd_url.make_relative(self) {
            Some(relative_url) => relative_url,
            None => self.as_str().to_owned(),
        }
    }
}

/// get the valid checks
/// invalid check are logged with error level and passed silently
/// todo: do not perform checks when at least one check has a definition error
fn get_checks_from_config_table(
    file_with_checks: &url::Url,
    file_to_check: PathBuf,
    config_table: &toml_edit::Table,
) -> Vec<Box<dyn Check>> {
    let mut checks = vec![];

    for (check_type, check_table) in config_table {
        match check_table {
            toml_edit::Item::Table(check_table) => {
                match get_check_from_check_table(
                    file_with_checks,
                    file_to_check.clone(),
                    check_type,
                    check_table,
                ) {
                    Ok(check) => checks.push(check),
                    Err(err) => {
                        log::error!("Checkfile {file_with_checks}:{check_type} has errors: {err}")
                    }
                }
            }
            toml_edit::Item::ArrayOfTables(array) => {
                for check_table in array {
                    match get_check_from_check_table(
                        file_with_checks,
                        file_to_check.clone(),
                        check_type,
                        check_table,
                    ) {
                        Ok(check) => checks.push(check),
                        Err(err) => log::error!(
                            "Checkfile {file_with_checks}:{check_type} has errors: {err}"
                        ),
                    }
                }
            }
            value => {
                log::error!(
                    "Unexpected value type {} {}",
                    value.type_name(),
                    file_with_checks.path()
                );
                std::process::exit(1);
            }
        };
    }
    checks
}

#[derive(Debug, Clone)]
pub(crate) struct GenericCheck {
    // path to the file where the checkers are defined
    pub(crate) file_with_checks: url::Url,
    // path to the file which needs to be checked
    pub(crate) file_to_check: PathBuf,
    // overridden file type
    pub(crate) file_type_override: Option<String>,
}

pub(crate) enum DefaultContent {
    None,
    EmptyString,
}
impl GenericCheck {
    fn file_with_checks(&self) -> &url::Url {
        &self.file_with_checks
    }

    fn file_to_check(&self) -> &PathBuf {
        &self.file_to_check
    }

    fn get_file_contents(&self, default_content: DefaultContent) -> Result<String, CheckError> {
        match fs::read_to_string(self.file_to_check()) {
            Ok(contents) => {
                let contents = if contents.ends_with_newline() {
                    contents
                } else {
                    format!("{contents}\n")
                };
                Ok(contents)
            }
            Err(e) => match default_content {
                DefaultContent::None => Err(CheckError::FileCanNotBeRead(e)),
                DefaultContent::EmptyString => Ok("".to_string()),
            },
        }
    }

    fn set_file_contents(&self, contents: String) -> Result<(), CheckError> {
        if fs::exists(self.file_to_check()).expect("no error checking existance of path")
            && contents.is_empty()
        {
            return Ok(());
        }

        if let Err(e) = fs::write(self.file_to_check(), contents) {
            log::error!(
                "⚠ Cannot write file {} {}",
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

        let contents = self.get_file_contents(DefaultContent::EmptyString)?;

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

fn determine_filetype_from_config_table(config_table: &mut toml_edit::Table) -> Option<String> {
    Some(
        config_table
            .remove("__filetype__")?
            .as_str()
            .expect("__filetype__ is a string")
            .to_string(),
    )
}

fn get_check_from_check_table(
    file_with_checks: &url::Url,
    file_to_check: PathBuf,
    check_type: &str,
    check_table: &toml_edit::Table,
) -> Result<Box<dyn Check>, CheckDefinitionError> {
    let mut check_table = check_table.clone();

    let generic_check = GenericCheck {
        file_with_checks: file_with_checks.clone(),
        file_to_check: file_to_check.clone(),
        file_type_override: determine_filetype_from_config_table(&mut check_table),
    };
    match check_type {
        "entry_absent" => Ok(Box::new(entry_absent::EntryAbsent::from_check_table(
            generic_check,
            check_table,
        )?)),
        "entry_present" => Ok(Box::new(entry_present::EntryPresent::from_check_table(
            generic_check,
            check_table,
        )?)),
        "file_absent" => Ok(Box::new(file_absent::FileAbsent::from_check_table(
            generic_check,
            check_table,
        )?)),
        "file_present" => Ok(Box::new(file_present::FilePresent::from_check_table(
            generic_check,
            check_table,
        )?)),
        "file_regex_match" => Ok(Box::new(file_regex::FileRegexMatch::from_check_table(
            generic_check,
            check_table,
        )?)),
        "lines_absent" => Ok(Box::new(lines_absent::LinesAbsent::from_check_table(
            generic_check,
            check_table,
        )?)),
        "lines_present" => Ok(Box::new(lines_present::LinesPresent::from_check_table(
            generic_check,
            check_table,
        )?)),
        "key_value_present" => Ok(Box::new(
            key_value_present::KeyValuePresent::from_check_table(
                generic_check,
                check_table.clone(),
            )?,
        )),
        "key_absent" => Ok(Box::new(key_absent::KeyAbsent::from_check_table(
            generic_check,
            check_table.clone(),
        )?)),
        "key_value_regex_match" => Ok(Box::new(
            key_value_regex_match::EntryRegexMatch::from_check_table(
                generic_check,
                check_table.clone(),
            )?,
        )),
        _ => {
            log::error!("unknown check {check_type} {check_table}");
            Err(CheckDefinitionError::UnknownCheckType(
                check_type.to_string(),
            ))
        }
    }
}

pub(crate) fn read_checks_from_path(
    file_with_checks: &url::Url,
    top_level_keys: Vec<&str>,
) -> Vec<Box<dyn Check>> {
    let mut checks: Vec<Box<dyn Check>> = vec![];

    let checks_toml_str = match uri::read_to_string(file_with_checks) {
        Ok(checks_toml) => checks_toml,
        Err(_) => {
            log::error!("⚠ {file_with_checks} could not be read");
            return checks;
        }
    };
    let mut checks_toml: toml_edit::Table =
        match toml_edit::DocumentMut::from_str(checks_toml_str.as_str()) {
            Ok(checks_toml) => checks_toml.as_table().to_owned(),
            Err(e) => {
                log::error!("Invalid toml file {file_with_checks} {e}");
                return checks;
            }
        };

    for key in top_level_keys {
        checks_toml = match checks_toml.get(key) {
            Some(toml) => match toml.as_table() {
                Some(toml) => toml.clone(),
                None => {
                    log::error!("Top level key {key} in {file_with_checks} is not a table");
                    return vec![];
                }
            },
            None => {
                log::error!("Top level key {key} is not found in {file_with_checks}");
                return vec![];
            }
        }
    }

    for (key, value) in checks_toml {
        // todo: is include possible as top level? So, without __config__ level
        if key == "__config__" {
            let value = value.get("include");
            if let Some(toml_edit::Item::Value(toml_edit::Value::Array(include_uris))) = value {
                for include_uri in include_uris {
                    let include_path = match uri::parse_uri(
                        include_uri.as_str().expect("uri is a string"),
                        Some(file_with_checks),
                    ) {
                        Ok(include_path) => include_path,
                        Err(_) => {
                            log::error!("{include_uri} is not a valid uri");
                            std::process::exit(1);
                        }
                    };
                    checks.extend(read_checks_from_path(&include_path, vec![]));
                }
            }
            continue;
        }

        let file_to_check = match expand_to_absolute(key.as_str()) {
            Ok(file) => file,
            Err(_) => {
                log::error!("path {key} can not be resolved");
                std::process::exit(1);
            }
        };
        match value {
            toml_edit::Item::Table(config_table) => {
                checks.extend(get_checks_from_config_table(
                    file_with_checks,
                    file_to_check.clone(),
                    &config_table,
                ));
            }
            toml_edit::Item::ArrayOfTables(array) => {
                for element in array {
                    checks.extend(get_checks_from_config_table(
                        file_with_checks,
                        file_to_check.clone(),
                        &element,
                    ));
                }
            }
            _ => {}
        }
    }
    checks
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_read_checks_from_path() {
        let dir = tempdir().unwrap();
        let path_with_checkers = dir.path().join("check-config.toml");
        let mut file_with_checkers = File::create(&path_with_checkers).unwrap();

        writeln!(
            file_with_checkers,
            r#"
[__config__]
include = []  # optional list of toml files with additional checks

["test/absent_file".file_absent]

["test/present_file".file_present]

["test/present.toml".key_absent.key]

["test/present.toml".key_value_present]
key1 = 1
key2 = "value"

["test/present.toml".key_value_regex_match]
key = 'v.*'

["test/present.txt".lines_absent]
__lines__ = """\
multi
line"""

["test/present.txt".lines_present]
__lines__ = """\
multi
line"""

["test/present.toml".entry_present.key]
__items__ = [1,2,3]

["test/present.toml".entry_absent.key]
__items__ = [1,2,3]
        "#
        )
        .unwrap();

        let path_with_checkers =
            url::Url::parse(&format!("file://{}", path_with_checkers.to_str().unwrap())).unwrap();
        let checks = read_checks_from_path(&path_with_checkers, vec![]);

        assert_eq!(checks.len(), 9);
    }

    #[test]
    fn test_read_invalid_checks_from_path() {
        let dir = tempdir().unwrap();
        let path_with_checkers = dir.path().join("check-config.toml");
        let mut file_with_checkers = File::create(&path_with_checkers).unwrap();

        writeln!(
            file_with_checkers,
            r#"
["test/absent_file".fileXabsent]

        "#
        )
        .expect("write is succsful");

        let path_with_checkers =
            url::Url::parse(&format!("file://{}", path_with_checkers.to_str().unwrap())).unwrap();
        let checks = read_checks_from_path(&path_with_checkers, vec![]);

        assert_eq!(checks.len(), 0);
    }
}
