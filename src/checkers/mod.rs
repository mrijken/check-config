use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::{env, str::FromStr};
use url::Url;

use base::CheckConstructor;

use crate::uri::{self};

use self::base::{CheckDefinitionError, Checker};

pub(crate) mod base;
pub(crate) mod file;
// pub(crate) mod package;
pub(crate) mod git;
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
    parent_tags: &Vec<String>,
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
                    parent_tags,
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
                        parent_tags,
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
pub(crate) struct GenericChecker {
    // path to the file where the checkers are defined
    pub(crate) file_with_checks: url::Url,
    // overridden file type
    pub(crate) tags: Vec<String>,
    // fixable
    pub(crate) fixable: bool,
}

impl GenericChecker {
    fn file_with_checks(&self) -> &url::Url {
        &self.file_with_checks
    }
}

fn tags_array_to_tags_vec(
    tags_array: &toml_edit::Array,
) -> Result<Vec<String>, CheckDefinitionError> {
    let mut tags = Vec::new();
    for i in tags_array {
        if let Some(value) = i.as_str() {
            tags.push(value.into());
        } else {
            return Err(CheckDefinitionError::InvalidDefinition(
                " __tags__ contains a value which is not a string".to_string(),
            ));
        };
    }

    Ok(tags)
}

fn read_tags_from_table(
    check_table: &toml_edit::Table,
) -> Result<Vec<String>, CheckDefinitionError> {
    match check_table.get("__tags__") {
        None => Ok(vec![]),
        Some(item) => {
            if !item.is_array() {
                Err(CheckDefinitionError::InvalidDefinition(
                    "`tags` is not an array".into(),
                ))
            } else {
                tags_array_to_tags_vec(item.as_array().expect("is an array"))
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
    file_with_checks: &url::Url,
    check_type: &str,
    check_table: &toml_edit::Table,
    parent_tags: &Vec<String>,
) -> Result<Box<dyn Check>, CheckDefinitionError> {
    let mut check_table = check_table.clone();

    let mut tags = read_tags_from_table(&check_table)?;
    for tag in parent_tags {
        tags.push(tag.clone());
    }

    let fixable = (get_option_boolean_from_check_table(&check_table, "fixable")?).unwrap_or(true);

    let generic_check = GenericChecker {
        file_with_checks: file_with_checks.clone(),
        tags,
        fixable,
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
        // "package_present" => Ok(Box::new(
        //     package::package_present::PackagePresent::from_check_table(generic_check, check_table)?,
        // )),
        // "package_absent" => Ok(Box::new(
        //     package::package_absent::PackageAbsent::from_check_table(generic_check, check_table)?,
        // )),
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
    file_with_checks: &url::Url,
    top_level_keys: Vec<&str>,
    parent_tags: &Vec<String>,
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

    let tags_of_this_file = if let Some(tags) = checks_toml.get("__tags__")
        && let toml_edit::Item::Value(toml_edit::Value::Array(tags)) = tags
    {
        match tags_array_to_tags_vec(tags) {
            Ok(mut tags) => {
                for tag in parent_tags {
                    if !tags.contains(tag) {
                        tags.push(tag.clone());
                    }
                }
                tags
            }
            Err(_) => {
                log::error!("{tags} contains invalid values");
                std::process::exit(1);
            }
        }
    } else {
        vec![]
    };

    if let Some(include) = checks_toml.get("__include__") {
        if let toml_edit::Item::Value(toml_edit::Value::Array(include_uris)) = include {
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
                checks.extend(read_checks_from_path(
                    &include_path,
                    vec![],
                    &tags_of_this_file,
                ));
            }
        }
    }

    for (key, value) in checks_toml {
        let file_to_check = match expand_to_absolute(key.as_str()) {
            Ok(file) => file,
            Err(_) => {
                log::error!("path {key} can not be resolved");
                std::process::exit(1);
            }
        };
        match value {
            toml_edit::Item::Table(config_table) => {
                checks_to_add.push(get_check_from_check_table(
                    file_with_checks,
                    check_type.as_str(),
                    &config_table,
                    &tags_of_this_file,
                ));
            }
            toml_edit::Item::ArrayOfTables(array) => {
                for config_table in array {
                    checks_to_add.push(get_check_from_check_table(
                        file_with_checks,
                        file_to_check.clone(),
                        &element,
                        &tags_of_this_file,
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

        let path_with_checkers =
            url::Url::parse(&format!("file://{}", path_with_checkers.to_str().unwrap())).unwrap();
        let checks = read_checks_from_path(&path_with_checkers, vec![], &vec![]);

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
        let checks = read_checks_from_path(&path_with_checkers, vec![], &vec![]);

        assert_eq!(checks.len(), 0);
    }
}
