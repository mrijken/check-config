use std::{env, fs, io, path::PathBuf};

use toml::Value;

use crate::file_types::{self, FileType};

use self::base::{Check, CheckError};

pub(crate) mod base;
pub(crate) mod file_absent;
pub(crate) mod file_present;
pub(crate) mod key_absent;
pub(crate) mod key_value_present;
pub(crate) mod key_value_regex_match;
pub(crate) mod lines_absent;
pub(crate) mod lines_present;

fn get_checks_from_config_table(
    file_with_checks: PathBuf,
    file_to_check: PathBuf,
    config_table: &toml::Table,
) -> Vec<Box<dyn Check>> {
    dbg!(&config_table, &file_with_checks);

    let mut checks = vec![];

    for (check_type, check_table) in config_table {
        match check_table {
            Value::Table(check_table) => {
                checks.push(get_check_from_check_table(
                    file_with_checks.clone(),
                    file_to_check.clone(),
                    check_type,
                    check_table,
                ));
            }
            Value::Array(array) => {
                for table in array {
                    let check_table = table.as_table().unwrap();
                    checks.push(get_check_from_check_table(
                        file_with_checks.clone(),
                        file_to_check.clone(),
                        check_type,
                        check_table,
                    ));
                }
            }
            _ => {
                panic!("Unexpected value type");
            }
        };
    }
    checks
}

#[derive(Debug)]
pub(crate) struct GenericCheck {
    // path to the file where the checkers are defined
    file_with_checks: PathBuf,
    // path to the file which needs to be checked
    file_to_check: PathBuf,
    // overriden file type
    file_type: Option<String>,
}

impl GenericCheck {
    fn file_with_checks(&self) -> &PathBuf {
        &self.file_with_checks
    }

    fn file_to_check(&self) -> &PathBuf {
        &self.file_to_check
    }

    fn get_file_contents(&self) -> io::Result<String> {
        fs::read_to_string(self.file_to_check())
    }

    /// Get the file type of the file_to_check
    fn file_type(&self) -> Result<Box<dyn FileType>, CheckError> {
        let extension = self.file_to_check().extension();
        if extension.is_none() {
            return Err(CheckError::UnknownFileType(
                "No extension found".to_string(),
            ));
        };

        let extension = self
            .file_type
            .clone()
            .unwrap_or(extension.unwrap().to_str().unwrap().to_string());

        if extension == "toml" {
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
        Err(CheckError::UnknownFileType(extension))
    }
}

fn determine_filetype_from_config_table(config_table: &mut toml::Table) -> Option<String> {
    Some(
        config_table
            .remove("__filetype__")?
            .as_str()
            .unwrap()
            .to_string(),
    )
}

fn get_check_from_check_table(
    file_with_checks: PathBuf,
    file_to_check: PathBuf,
    check_type: &str,
    check_table: &toml::Table,
) -> Box<dyn Check> {
    let mut check_table = check_table.clone();

    let generic_check = GenericCheck {
        file_with_checks: file_with_checks.clone(),
        file_to_check: file_to_check.clone(),
        file_type: determine_filetype_from_config_table(&mut check_table),
    };
    let check: Box<dyn Check> = match check_type {
        "file_absent" => Box::new(file_absent::FileAbsent::new(generic_check)),
        "file_present" => Box::new(file_present::FilePresent::new(generic_check)),
        "lines_absent" => Box::new(lines_absent::LinesAbsent::new(
            generic_check,
            check_table
                .get("__lines__")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
        )),
        "lines_present" => Box::new(lines_present::LinesPresent::new(
            generic_check,
            check_table
                .get("__lines__")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
        )),
        "key_value_present" => Box::new(key_value_present::KeyValuePresent::new(
            generic_check,
            check_table.clone(),
        )),
        "key_absent" => Box::new(key_absent::KeyAbsent::new(
            generic_check,
            check_table.clone(),
        )),
        "key_value_regex_match" => Box::new(key_value_regex_match::EntryRegexMatch::new(
            generic_check,
            check_table.clone(),
        )),
        _ => panic!("unknown check {} {}", check_type, check_table),
    };
    check
}

pub(crate) fn read_checks_from_path(file_with_checks: &PathBuf) -> Vec<Box<dyn Check>> {
    let mut checks: Vec<Box<dyn Check>> = vec![];

    if !file_with_checks.exists() {
        log::error!("{} does not exist", file_with_checks.to_string_lossy());
        return checks;
    }

    let checks_toml = fs::read_to_string(file_with_checks).unwrap();
    let checks_toml: toml::Table = toml::from_str(checks_toml.as_str()).unwrap();

    for (file_to_check, value) in checks_toml {
        if file_to_check == "check-config" {
            if let Some(Value::Array(includes)) = value.get("additional_checks") {
                for include_path in includes {
                    checks.extend(read_checks_from_path(
                        &file_with_checks
                            .parent()
                            .unwrap()
                            .join(include_path.as_str().unwrap()),
                    ))
                }
            }
            continue;
        }
        let file_to_check = env::current_dir().unwrap().join(file_to_check);
        match value {
            Value::Table(config_table) => {
                checks.extend(get_checks_from_config_table(
                    file_with_checks.clone(),
                    file_to_check,
                    &config_table,
                ));
            }
            Value::Array(array) => {
                for element in array {
                    if let Some(config_table) = element.as_table() {
                        checks.extend(get_checks_from_config_table(
                            file_with_checks.clone(),
                            file_to_check.clone(),
                            config_table,
                        ));
                    }
                }
            }
            _ => {}
        }
    }
    checks
}
