use std::{env, fs, path::PathBuf};

use toml::Value;

use self::base::Check;

pub(crate) mod base;
pub(crate) mod entry_absent;
pub(crate) mod entry_present;
pub(crate) mod file_absent;
pub(crate) mod file_present;
pub(crate) mod lines_absent;
pub(crate) mod lines_present;

fn get_checks_from_config_table(
    checkers_path: PathBuf,
    config_path: PathBuf,
    config_table: &toml::Table,
) -> Vec<Box<dyn Check>> {
    dbg!(&config_table, &checkers_path);

    let mut checks = vec![];

    for (check_type, check_table) in config_table {
        match check_table {
            Value::Table(check_table) => {
                checks.push(get_check_from_check_table(
                    checkers_path.clone(),
                    config_path.clone(),
                    check_type,
                    check_table,
                ));
            }
            Value::Array(array) => {
                for table in array {
                    let check_table = table.as_table().unwrap();
                    checks.push(get_check_from_check_table(
                        checkers_path.clone(),
                        config_path.clone(),
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
    checkers_path: PathBuf,
    config_path: PathBuf,
    check_type: &str,
    check_table: &toml::Table,
) -> Box<dyn Check> {
    let checkers_path = checkers_path.clone();
    let config_path = config_path.clone();
    let check: Box<dyn Check> = match check_type {
        "file_absent" => Box::new(file_absent::FileAbsent::new(checkers_path, config_path)),
        "file_present" => Box::new(file_present::FilePresent::new(checkers_path, config_path)),
        "lines_absent" => Box::new(lines_absent::LinesAbsent::new(
            checkers_path,
            config_path,
            check_table
                .get("__lines__")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
        )),
        "lines_present" => Box::new(lines_present::LinesPresent::new(
            checkers_path,
            config_path,
            check_table
                .get("__lines__")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
        )),
        "entry_present" => Box::new(entry_present::EntryPresent::new(
            checkers_path,
            config_path,
            check_table.clone(),
        )),
        "entry_absent" => Box::new(entry_absent::EntryAbsent::new(
            checkers_path.clone(),
            config_path,
            check_table.clone(),
        )),
        _ => panic!("unknown check {} {}", check_type, check_table),
    };
    check
}

pub(crate) fn read_checks_from_path(
    checkers_path: &PathBuf,
) -> Result<Vec<Box<dyn Check>>, String> {
    if !checkers_path.exists() {
        return Err(format!(
            "{} does not exist",
            checkers_path.to_string_lossy()
        ));
    }

    let checks_toml = fs::read_to_string(checkers_path).unwrap();
    let checks_toml: toml::Table = toml::from_str(checks_toml.as_str()).unwrap();

    let mut checks: Vec<Box<dyn Check>> = vec![];

    for (config_path, value) in checks_toml {
        if config_path == "check-config" {
            if let Some(Value::Array(includes)) = value.get("additional_checks") {
                for include_path in includes {
                    checks.extend(read_checks_from_path(
                        &checkers_path
                            .parent()
                            .unwrap()
                            .join(include_path.as_str().unwrap()),
                    )?)
                }
            }
            continue;
        }
        let config_path = env::current_dir().unwrap().join(config_path);
        match value {
            Value::Table(config_table) => {
                checks.extend(get_checks_from_config_table(
                    checkers_path.clone(),
                    config_path,
                    &config_table,
                ));
            }
            Value::Array(array) => {
                for element in array {
                    if let Some(config_table) = element.as_table() {
                        checks.extend(get_checks_from_config_table(
                            checkers_path.clone(),
                            config_path.clone(),
                            config_table,
                        ));
                    }
                }
            }
            _ => {}
        }
    }
    Ok(checks)
}
