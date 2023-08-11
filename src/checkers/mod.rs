use std::{env, fs, path::PathBuf};

use toml::Value;

use self::base::Check;

pub(crate) mod base;
pub(crate) mod entry_regex_match;
pub(crate) mod file_absent;
pub(crate) mod file_present;
pub(crate) mod key_absent;
pub(crate) mod key_value_present;
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

fn get_check_from_check_table(
    file_with_checks: PathBuf,
    file_to_check: PathBuf,
    check_type: &str,
    check_table: &toml::Table,
) -> Box<dyn Check> {
    let file_with_checks = file_with_checks.clone();
    let file_to_check = file_to_check.clone();
    let check: Box<dyn Check> = match check_type {
        "file_absent" => Box::new(file_absent::FileAbsent::new(
            file_with_checks,
            file_to_check,
        )),
        "file_present" => Box::new(file_present::FilePresent::new(
            file_with_checks,
            file_to_check,
        )),
        "lines_absent" => Box::new(lines_absent::LinesAbsent::new(
            file_with_checks,
            file_to_check,
            check_table
                .get("__lines__")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
        )),
        "lines_present" => Box::new(lines_present::LinesPresent::new(
            file_with_checks,
            file_to_check,
            check_table
                .get("__lines__")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
        )),
        "key_value_present" => Box::new(key_value_present::KeyValuePresent::new(
            file_with_checks,
            file_to_check,
            check_table.clone(),
        )),
        "key_absent" => Box::new(key_absent::KeyAbsent::new(
            file_with_checks.clone(),
            file_to_check,
            check_table.clone(),
        )),
        "entry_regex_match" => Box::new(entry_regex_match::EntryRegexMatch::new(
            file_with_checks,
            file_to_check,
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
