use std::{collections::HashMap, env, str::FromStr};

use base::CheckConstructor;
use url::Url;

use crate::uri::{ReadPath, ReadablePath};

use self::base::{CheckDefinitionError, Checker};

pub(crate) mod base;
pub(crate) mod file;
pub(crate) mod git;
pub(crate) mod package;
pub(crate) mod test_helpers;
pub(crate) mod utils;

pub(crate) trait RelativeUrl {
    fn short_url_str(&self) -> String;
}

impl RelativeUrl for ReadablePath {
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
        match cwd_url.make_relative(self.as_ref()) {
            Some(relative_url) => relative_url,
            None => self.as_ref().as_str().to_owned(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct GenericChecker {
    // path to the file where the checkers are defined
    pub(crate) file_with_checks: ReadablePath,
    // overridden file type
    pub(crate) tags: Vec<String>,
    // check_only
    pub(crate) check_only: bool,
    // variables which are present and can be used for templating
    // this is a owned hashmap to make sure that only variables
    // which are read before the definition of this checker are used
    pub(crate) variables: HashMap<String, String>,
}

impl GenericChecker {
    fn file_with_checks(&self) -> &ReadablePath {
        &self.file_with_checks
    }
}

fn read_tags_from_table(
    check_table: &toml_edit::Table,
) -> Result<Vec<String>, CheckDefinitionError> {
    let mut tags = Vec::new();
    match check_table.get("tags") {
        None => Ok(tags),
        Some(item) => {
            if !item.is_array() {
                Err(CheckDefinitionError::InvalidDefinition(
                    "`tags` is not an array".into(),
                ))
            } else {
                for i in item.as_array().unwrap() {
                    if let Some(value) = i.as_str() {
                        tags.push(value.into());
                    } else {
                        return Err(CheckDefinitionError::InvalidDefinition(
                            "`tags` contains a value which is not a string".to_string(),
                        ));
                    };
                }

                Ok(tags)
            }
        }
    }
}

fn get_option_boolean_from_check_table(
    check_table: &toml_edit::Table,
    key: &str,
) -> Result<Option<bool>, CheckDefinitionError> {
    match check_table.get(key) {
        None => Ok(None),
        Some(value) => match value.as_bool() {
            Some(value) => Ok(Some(value)),
            None => Err(CheckDefinitionError::InvalidDefinition(format!(
                "{key} is not a boolean",
            ))),
        },
    }
}

fn get_check_from_check_table(
    file_with_checks: &ReadablePath,
    check_type: &str,
    check_table: &toml_edit::Table,
    variables: HashMap<String, String>,
) -> Result<Box<dyn Checker>, CheckDefinitionError> {
    let check_table = check_table.clone();

    let tags = read_tags_from_table(&check_table)?;

    let check_only =
        (get_option_boolean_from_check_table(&check_table, "check_only")?).unwrap_or(false);

    let generic_check = GenericChecker {
        file_with_checks: file_with_checks.clone(),
        tags,
        check_only,
        variables,
    };
    match check_type {
        "entry_absent" => Ok(Box::new(file::entry_absent::EntryAbsent::from_check_table(
            generic_check,
            check_table,
        )?)),
        "entry_present" => Ok(Box::new(
            file::entry_present::EntryPresent::from_check_table(generic_check, check_table)?,
        )),
        "file_absent" => Ok(Box::new(file::file_absent::FileAbsent::from_check_table(
            generic_check,
            check_table,
        )?)),
        "file_present" => Ok(Box::new(file::file_present::FilePresent::from_check_table(
            generic_check,
            check_table,
        )?)),
        "file_copied" => Ok(Box::new(file::file_copied::FileCopied::from_check_table(
            generic_check,
            check_table,
        )?)),
        "dir_copied" => Ok(Box::new(file::dir_copied::DirCopied::from_check_table(
            generic_check,
            check_table,
        )?)),
        "dir_present" => Ok(Box::new(file::dir_present::DirPresent::from_check_table(
            generic_check,
            check_table,
        )?)),
        "dir_absent" => Ok(Box::new(file::dir_absent::DirAbsent::from_check_table(
            generic_check,
            check_table,
        )?)),
        "file_unpacked" => Ok(Box::new(
            file::file_unpacked::FileUnpacked::from_check_table(generic_check, check_table)?,
        )),
        "lines_absent" => Ok(Box::new(file::lines_absent::LinesAbsent::from_check_table(
            generic_check,
            check_table,
        )?)),
        "lines_present" => Ok(Box::new(
            file::lines_present::LinesPresent::from_check_table(generic_check, check_table)?,
        )),
        "package_present" => Ok(Box::new(
            package::package_present::PackagePresent::from_check_table(generic_check, check_table)?,
        )),
        "package_absent" => Ok(Box::new(
            package::package_absent::PackageAbsent::from_check_table(generic_check, check_table)?,
        )),
        "key_value_present" => Ok(Box::new(
            file::key_value_present::KeyValuePresent::from_check_table(
                generic_check,
                check_table.clone(),
            )?,
        )),
        "key_absent" => Ok(Box::new(file::key_absent::KeyAbsent::from_check_table(
            generic_check,
            check_table.clone(),
        )?)),
        "key_value_regex_matched" => Ok(Box::new(
            file::key_value_regex_match::EntryRegexMatched::from_check_table(
                generic_check,
                check_table.clone(),
            )?,
        )),
        "git_fetched" => Ok(Box::new(git::GitFetched::from_check_table(
            generic_check,
            check_table.clone(),
        )?)),
        _ => {
            log::error!("unknown check {check_type} {check_table}");
            Err(CheckDefinitionError::UnknownCheckType(
                check_type.to_string(),
            ))
        }
    }
}

pub(crate) fn read_checks_from_path(
    file_with_checks: &ReadablePath,
    variables: &mut HashMap<String, String>,
) -> Vec<Box<dyn Checker>> {
    let mut checks: Vec<Box<dyn Checker>> = vec![];
    let mut file_with_checks = file_with_checks.clone();
    let checks_toml_str = match file_with_checks.read_to_string() {
        Ok(checks_toml) => checks_toml,
        Err(_) => {
            let uri = match Url::parse(
                format!("{}/check-config.toml", file_with_checks.as_ref()).as_str(),
            ) {
                Ok(uri) => uri,
                Err(_) => {
                    log::error!("⚠ {file_with_checks} could not be read");
                    return checks;
                }
            };
            file_with_checks = ReadablePath::from_url(uri);
            match file_with_checks.read_to_string() {
                Ok(checks_toml) => checks_toml,
                Err(_) => {
                    log::error!("⚠ {file_with_checks} could not be read");
                    return checks;
                }
            }
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

    let top_level_keys = if file_with_checks.as_ref().path().ends_with("pyproject.toml") {
        vec!["tool", "check-config"]
    } else if file_with_checks.as_ref().path().ends_with("Cargo.toml") {
        vec!["package", "metadata", "check-config"]
    } else {
        vec![]
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
        if key == "include" {
            if let toml_edit::Item::Value(toml_edit::Value::Array(include_uris)) = value {
                for include_uri in include_uris {
                    let include_path = match ReadablePath::from_string(
                        include_uri.as_str().expect("uri is a string"),
                        Some(&file_with_checks),
                    ) {
                        Ok(include_path) => include_path,
                        Err(_) => {
                            log::error!("{include_uri} is not a valid uri");
                            std::process::exit(1);
                        }
                    };
                    checks.extend(read_checks_from_path(&include_path, variables));
                }
            }

            continue;
        }
        if key == "variables" {
            if let toml_edit::Item::Table(current_variables) = &value {
                current_variables.iter().for_each(|(k, v)| {
                    let v = v.as_str().expect("value is a string");
                    // TODO: fix error or convert when value is not a string
                    variables.insert(k.to_string(), v.to_string());
                });
            }

            continue;
        }

        let check_type = key;
        let mut checks_to_add = vec![];
        match value {
            toml_edit::Item::Table(config_table) => {
                checks_to_add.push(get_check_from_check_table(
                    &file_with_checks,
                    check_type.as_str(),
                    &config_table,
                    variables.clone(),
                ));
            }
            toml_edit::Item::ArrayOfTables(array) => {
                for config_table in array {
                    checks_to_add.push(get_check_from_check_table(
                        &file_with_checks,
                        check_type.as_str(),
                        &config_table,
                        variables.clone(),
                    ));
                }
            }
            _ => {}
        }

        for check in checks_to_add {
            match check {
                Ok(check) => checks.push(check),
                Err(err) => {
                    log::error!("Checkfile {file_with_checks}:{check_type} has errors: {err}")
                }
            }
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
include = []  # optional list of toml files with additional checks

[[file_absent]]
file = "test/absent_file"

[[file_present]]
file = "test/present_file"

[[key_absent]]
file = "test/present.toml"
key.key = "key"

[[key_value_present]]
file = "test/present.toml"
key.key1 = 1

[key_value_regex_matched]
file = "test/present.toml"
key.key = 'v.*'

[[lines_absent]]
file = "test/present.txt"
lines = """\
multi
line"""

[[lines_present]]
file = "test/present.txt"
lines = """\
multi
line"""

[[entry_present]]
file = "test/present.toml"
entry.key = [1,2,3]

[[entry_absent]]
file = "test/present.toml"
entry.key = [1,2,3]
        "#
        )
        .expect("file is created");

        let mut variables = HashMap::new();
        let path_with_checkers = ReadablePath::from_string(
            &format!("file://{}", path_with_checkers.to_str().unwrap()),
            None,
        )
        .unwrap();
        let checks = read_checks_from_path(&path_with_checkers, &mut variables);

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

        let mut variables = HashMap::new();

        let path_with_checkers = ReadablePath::from_string(
            &format!("file://{}", path_with_checkers.to_str().unwrap()),
            None,
        )
        .unwrap();
        let checks = read_checks_from_path(&path_with_checkers, &mut variables);

        assert_eq!(checks.len(), 0);
    }
}
