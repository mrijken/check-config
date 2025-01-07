use std::{
    env,
    fs::{self},
    path::PathBuf,
};

use toml::Value;

use crate::{
    file_types::{self, FileType},
    mapping::generic::Mapping,
    uri::Uri,
};

use self::base::{Check, CheckError};

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

fn get_checks_from_config_table(
    file_with_checks: &Uri,
    file_to_check: PathBuf,
    config_table: &toml::Table,
) -> Vec<Box<dyn Check>> {
    let mut checks = vec![];

    for (check_type, check_table) in config_table {
        match check_table {
            Value::Table(check_table) => {
                checks.push(get_check_from_check_table(
                    file_with_checks,
                    file_to_check.clone(),
                    check_type,
                    check_table,
                ));
            }
            Value::Array(array) => {
                for table in array {
                    let check_table = table.as_table().expect("value is a table");
                    checks.push(get_check_from_check_table(
                        file_with_checks,
                        file_to_check.clone(),
                        check_type,
                        check_table,
                    ));
                }
            }
            _ => {
                log::error!("Unexpected value type");
                std::process::exit(1);
            }
        };
    }
    checks
}

#[derive(Debug, Clone)]
pub(crate) struct GenericCheck {
    // path to the file where the checkers are defined
    file_with_checks: Uri,
    // path to the file which needs to be checked
    file_to_check: PathBuf,
    // overridden file type
    file_type_override: Option<String>,
}

pub(crate) enum DefaultContent {
    None,
    EmptyString,
}
impl GenericCheck {
    fn file_with_checks(&self) -> &Uri {
        &self.file_with_checks
    }

    fn file_to_check(&self) -> &PathBuf {
        &self.file_to_check
    }

    fn get_file_contents(&self, default_content: DefaultContent) -> Result<String, CheckError> {
        match fs::read_to_string(self.file_to_check()) {
            Ok(contents) => Ok(contents),
            Err(e) => match default_content {
                DefaultContent::None => Err(CheckError::FileCanNotBeRead(e)),
                DefaultContent::EmptyString => Ok("".to_string()),
            },
        }
    }

    fn set_file_contents(&self, contents: String) -> Result<(), CheckError> {
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

fn determine_filetype_from_config_table(config_table: &mut toml::Table) -> Option<String> {
    Some(
        config_table
            .remove("__filetype__")?
            .as_str()
            .expect("__filetype__ is a string")
            .to_string(),
    )
}

fn get_check_from_check_table(
    file_with_checks: &Uri,
    file_to_check: PathBuf,
    check_type: &str,
    check_table: &toml::Table,
) -> Box<dyn Check> {
    let mut check_table = check_table.clone();

    let generic_check = GenericCheck {
        file_with_checks: file_with_checks.clone(),
        file_to_check: file_to_check.clone(),
        file_type_override: determine_filetype_from_config_table(&mut check_table),
    };
    let check: Box<dyn Check> = match check_type {
        "entry_absent" => Box::new(entry_absent::EntryAbsent::new(
            generic_check,
            check_table.clone(),
        )),
        "entry_present" => Box::new(entry_present::EntryPresent::new(
            generic_check,
            check_table.clone(),
        )),
        "file_absent" => Box::new(file_absent::FileAbsent::new(generic_check)),
        "file_present" => Box::new(file_present::FilePresent::new(generic_check)),
        "file_regex_match" => {
            if check_table.get("__regex__").is_none() {
                log::error!("No __regex__ found in {}", check_table);
                std::process::exit(1);
            }
            Box::new(file_regex::FileRegexMatch::new(
                generic_check,
                check_table
                    .get("__regex__")
                    .expect("__regex__ is present")
                    .as_str()
                    .expect("__regex__ is a string")
                    .to_string(),
            ))
        }
        "lines_absent" => {
            if check_table.get("__lines__").is_none() {
                log::error!("No __lines__ found in {}", check_table);
                std::process::exit(1);
            }
            Box::new(lines_absent::LinesAbsent::new(
                generic_check,
                check_table
                    .get("__lines__")
                    .expect("__lines__ is present")
                    .as_str()
                    .expect("__lines__ is a string")
                    .to_string(),
            ))
        }
        "lines_present" => {
            if check_table.get("__lines__").is_none() {
                log::error!("No __lines__ found in {}", check_table);
                std::process::exit(1);
            }
            Box::new(lines_present::LinesPresent::new(
                generic_check,
                check_table
                    .get("__lines__")
                    .expect("__lines__ is present")
                    .as_str()
                    .expect("__lines__ is a string")
                    .to_string(),
            ))
        }
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
        _ => {
            log::error!("unknown check {} {}", check_type, check_table);

            // exit can not be tested
            #[cfg(test)]
            core::panic!("unknown check");

            #[cfg(not(test))]
            std::process::exit(1);
        }
    };
    check
}

pub(crate) fn read_checks_from_path(file_with_checks: &Uri) -> Vec<Box<dyn Check>> {
    let mut checks: Vec<Box<dyn Check>> = vec![];

    let checks_toml = match file_with_checks.read_to_string() {
        Ok(checks_toml) => checks_toml,
        Err(_) => {
            log::error!("⚠ {} could not be read", file_with_checks);
            return checks;
        }
    };
    let checks_toml: toml::Table = toml::from_str(checks_toml.as_str()).expect("valid toml");

    for (file_to_check, value) in checks_toml {
        if file_to_check == "check-config" {
            if let Some(Value::Array(include_uris)) = value.get("additional_checks") {
                for include_uri in include_uris {
                    let include_path =
                        match Uri::new(include_uri.as_str().expect("uri is a string")) {
                            Ok(include_path) => include_path,
                            Err(_) => match file_with_checks
                                .join(include_uri.as_str().expect("uri is a string"))
                            {
                                Ok(include_path) => include_path,
                                Err(_) => {
                                    log::error!("{} is not a valid uri", include_uri);
                                    std::process::exit(1);
                                }
                            },
                        };
                    checks.extend(read_checks_from_path(&include_path));
                }
            }
            continue;
        }
        let file_to_check = env::current_dir()
            .expect("current dir exists")
            .join(file_to_check);
        match value {
            Value::Table(config_table) => {
                checks.extend(get_checks_from_config_table(
                    file_with_checks,
                    file_to_check.clone(),
                    &config_table,
                ));
            }
            Value::Array(array) => {
                for element in array {
                    if let Some(config_table) = element.as_table() {
                        checks.extend(get_checks_from_config_table(
                            file_with_checks,
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

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_read_checks_from_path() {
        let dir = tempdir().unwrap();
        let path_with_checkers = dir.path().join("checkers.toml");
        let mut file_with_checkers = File::create(&path_with_checkers).unwrap();

        writeln!(
            file_with_checkers,
            r#"
[check-config]
additional_checks = []  # optional list of toml files with additional checks

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

        let path_with_checkers = crate::uri::Uri::Path(path_with_checkers);
        let checks = read_checks_from_path(&path_with_checkers);

        assert_eq!(checks.len(), 9);
    }

    #[test]
    #[should_panic]
    fn test_read_invalid_checks_from_path() {
        let dir = tempdir().unwrap();
        let path_with_checkers = dir.path().join("checkers.toml");
        let mut file_with_checkers = File::create(&path_with_checkers).unwrap();

        writeln!(
            file_with_checkers,
            r#"
["test/absent_file".fileXabsent]

        "#
        )
        .unwrap();

        let path_with_checkers = crate::uri::Uri::Path(path_with_checkers);
        let checks = read_checks_from_path(&path_with_checkers);

        assert_eq!(checks.len(), 0);
    }
}
