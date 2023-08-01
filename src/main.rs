use core::fmt::Debug as DebugTrait;
use std::{
    env,
    fs::{self, OpenOptions},
    path::PathBuf,
};

mod checkers;
use similar::TextDiff;
use toml::Value;

use clap::{Parser, Subcommand};

/// Config Checker will check and optional fix your config files based on checkers defined in a toml file.
/// It can check ini, toml, yaml, json and plain text files.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the root checkers file in toml format
    #[arg(short, long, default_value = "checkers.toml")]
    checkers_path: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Try to fix the files so the checkers will not fail anymore.
    /// Will return with exit code 1 when failed
    Fix,
    /// Check the style without checking.
    /// Will return with exit code 1 when failed
    Check,
}

fn main() -> Result<(), String> {
    simple_logger::init().unwrap();
    log::info!("Starting config-checker");

    let cli = Cli::parse();

    let perform_fix = match &cli.command {
        Commands::Fix => {
            log::info!("With check and fix");
            true
        }
        Commands::Check => {
            log::info!("Only check");
            false
        }
    };

    let checkers_path = &PathBuf::from(&cli.checkers_path);

    log::info!("Using checkers from {}", &checkers_path.to_string_lossy());

    let checks = read_checks_from_path(checkers_path)?;

    let mut is_all_ok = true;

    for check in checks {
        match check.check() {
            Ok(ist_and_soll) => match ist_and_soll.action {
                Action::None => {
                    log::info!(
                        "✅ {} - {} - {}",
                        check.checkers_path().to_string_lossy(),
                        check.config_path().to_string_lossy(),
                        check.style_type(),
                    );
                }
                Action::RemoveFile => {
                    log::error!(
                        "❌ {} - {} - {} - file is present",
                        check.checkers_path().to_string_lossy(),
                        check.config_path().to_string_lossy(),
                        check.style_type(),
                    );
                    is_all_ok = perform_fix;
                }
                Action::SetContents => {
                    log::error!(
                        "❌ {} - {} - {} - diff",
                        check.checkers_path().to_string_lossy(),
                        check.config_path().to_string_lossy(),
                        check.style_type(),
                    );
                    log::info!(
                        "{}",
                        TextDiff::from_lines(&ist_and_soll.ist, &ist_and_soll.soll).unified_diff()
                    );
                    is_all_ok = perform_fix;
                }
            },
            Err(e) => {
                log::error!(
                    "Error: {} {} {} {}",
                    e,
                    check.checkers_path().to_string_lossy(),
                    check.config_path().to_string_lossy(),
                    check.style_type(),
                );
                is_all_ok = false;
            }
        }

        if perform_fix && check.fix().is_err() {
            is_all_ok = false;
        }
    }

    if is_all_ok {
        Ok(())
    } else {
        Err("One or more errors occured".to_string())
    }
}

fn read_checks_from_path(checkers_path: &PathBuf) -> Result<Vec<Box<dyn Check>>, String> {
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
        if config_path == "nitpick" {
            if let Some(Value::Array(includes)) = value.get("additional_styles") {
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
        "file_absent" => Box::new(FileAbsent {
            checkers_path,
            config_path,
        }),
        "file_present" => Box::new(FilePresent {
            checkers_path,
            config_path,
        }),
        "lines_absent" => Box::new(LinesAbsent {
            checkers_path,
            config_path,
            block: check_table
                .get("__lines__")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
        }),
        "lines_present" => Box::new(LinesPresent {
            checkers_path,
            config_path,
            lines: check_table
                .get("__lines__")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
        }),
        "entry_present" => Box::new(EntryPresent {
            checkers_path,
            config_path,
            value: check_table.clone(),
        }),
        "entry_absent" => Box::new(EntryAbsent {
            checkers_path: checkers_path.clone(),
            config_path,
            value: check_table.clone(),
        }),
        _ => panic!("unknown check {} {}", check_type, check_table),
    };
    check
}
enum Action {
    RemoveFile,
    SetContents,
    None,
}
struct IstAndSoll {
    ist: String,
    soll: String,
    action: Action,
}

trait Check: DebugTrait {
    fn style_type(&self) -> String;
    fn get_ist_and_soll(&self) -> Result<IstAndSoll, String>;
    fn checkers_path(&self) -> &PathBuf;
    fn config_path(&self) -> &PathBuf;

    fn check(&self) -> Result<IstAndSoll, String> {
        self.get_ist_and_soll()
    }

    fn fix(&self) -> Result<(), String> {
        let ist_and_soll = self.get_ist_and_soll()?;
        match ist_and_soll.action {
            Action::RemoveFile => match fs::remove_file(self.config_path()) {
                Ok(()) => Ok(()),
                Err(_) => Err("Cannot remove file".to_string()),
            },
            Action::SetContents => match fs::write(self.config_path(), ist_and_soll.soll) {
                Ok(()) => Ok(()),
                Err(_) => Err("Cannot write file".to_string()),
            },
            Action::None => Ok(()),
        }
    }
}

#[derive(Debug)]
struct LinesPresent {
    // path to the file where the checkers are defined
    checkers_path: PathBuf,
    // path to the file which needs to be checked
    config_path: PathBuf,
    lines: String,
}

impl Check for LinesPresent {
    fn style_type(&self) -> String {
        "lines_present".to_string()
    }

    fn checkers_path(&self) -> &PathBuf {
        &self.checkers_path
    }

    fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    fn get_ist_and_soll(&self) -> Result<IstAndSoll, String> {
        if !self.config_path().exists() {
            return Ok(IstAndSoll {
                ist: "".to_string(),
                soll: self.lines.clone(),
                action: Action::SetContents,
            });
        }
        match fs::read_to_string(self.config_path()) {
            Ok(contents) => {
                if contents.contains(&self.lines) {
                    Ok(IstAndSoll {
                        ist: contents.clone(),
                        soll: contents.clone(),
                        action: Action::None,
                    })
                } else {
                    let mut new_contents = contents.clone();
                    if !new_contents.ends_with('\n') {
                        new_contents += "\n";
                    }
                    new_contents += &self.lines.clone();
                    Ok(IstAndSoll {
                        ist: contents.clone(),
                        soll: new_contents,
                        action: Action::SetContents,
                    })
                }
            }
            Err(err) => Err(err.to_string()),
        }
    }
}

#[derive(Debug)]
struct LinesAbsent {
    // path to the file where the checkers are defined
    checkers_path: PathBuf,
    // path to the file which needs to be checked
    config_path: PathBuf,
    block: String,
}
impl Check for LinesAbsent {
    fn style_type(&self) -> String {
        "lines_absent".to_string()
    }

    fn checkers_path(&self) -> &PathBuf {
        &self.checkers_path
    }

    fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    fn get_ist_and_soll(&self) -> Result<IstAndSoll, String> {
        if !self.config_path().exists() {
            return Ok(IstAndSoll {
                ist: "".to_string(),
                soll: "".to_string(),
                action: Action::None,
            });
        }

        match fs::read_to_string(self.config_path()) {
            Ok(contents) => {
                if contents.contains(&self.block) {
                    let new_contents = contents.replace(&self.block, "");
                    Ok(IstAndSoll {
                        ist: contents,
                        soll: new_contents,
                        action: Action::SetContents,
                    })
                } else {
                    Ok(IstAndSoll {
                        ist: contents.clone(),
                        soll: contents,
                        action: Action::None,
                    })
                }
            }
            Err(_) => Err("error performing check".to_string()),
        }
    }

    fn fix(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug)]
struct FileAbsent {
    // path to the file where the checkers are defined
    checkers_path: PathBuf,
    // path to the file which needs to be checked
    config_path: PathBuf,
}

impl Check for FileAbsent {
    fn style_type(&self) -> String {
        "file_absent".to_string()
    }

    fn checkers_path(&self) -> &PathBuf {
        &self.checkers_path
    }

    fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    fn get_ist_and_soll(&self) -> Result<IstAndSoll, String> {
        match self.config_path.exists() {
            true => Ok(IstAndSoll {
                ist: "".to_string(),
                soll: "".to_string(),
                action: Action::RemoveFile,
            }),
            false => Ok(IstAndSoll {
                ist: "".to_string(),
                soll: "".to_string(),
                action: Action::None,
            }),
        }
    }

    fn fix(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug)]
struct FilePresent {
    // path to the file where the checkers are defined
    checkers_path: PathBuf,
    // path to the file which needs to be checked
    config_path: PathBuf,
}

impl Check for FilePresent {
    fn style_type(&self) -> String {
        "file_present".to_string()
    }

    fn checkers_path(&self) -> &PathBuf {
        &self.checkers_path
    }

    fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    fn get_ist_and_soll(&self) -> Result<IstAndSoll, String> {
        match self.config_path.exists() {
            false => Ok(IstAndSoll {
                ist: "".to_string(),
                soll: "".to_string(),
                action: Action::SetContents,
            }),
            true => Ok(IstAndSoll {
                ist: "".to_string(),
                soll: "".to_string(),
                action: Action::None,
            }),
        }
    }

    fn fix(&self) -> Result<(), String> {
        match OpenOptions::new()
            .create(true)
            .write(true)
            .open(&self.config_path)
        {
            Ok(_) => Ok(()),
            Err(err) => Err(err.to_string()),
        }
    }
}

#[derive(Debug)]
pub struct EntryPresent {
    // path to the file where the checkers are defined
    checkers_path: PathBuf,
    // path to the file which needs to be checked
    config_path: PathBuf,
    value: toml::Table,
}

impl EntryPresent {
    fn new(checkers_path: PathBuf, config_path: PathBuf, value: toml::Table) -> Self {
        Self {
            checkers_path,
            config_path,
            value,
        }
    }
}

use toml_edit::{value, Document, Item};

impl Check for EntryPresent {
    fn style_type(&self) -> String {
        "entry_present".to_string()
    }

    fn checkers_path(&self) -> &PathBuf {
        &self.checkers_path
    }

    fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    fn get_ist_and_soll(&self) -> Result<IstAndSoll, String> {
        let contents = if !self.config_path().exists() {
            "".to_string()
        } else {
            let contents = fs::read_to_string(self.config_path());
            if let Err(s) = contents {
                return Err(s.to_string());
            }
            contents.unwrap()
        };

        let new_contents = checkers::toml::set(&contents, &self.value).unwrap();

        let action = if contents == new_contents {
            Action::None
        } else {
            Action::SetContents
        };
        Ok(IstAndSoll {
            ist: contents,
            soll: new_contents,
            action,
        })
    }

    fn fix(&self) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct EntryAbsent {
    // path to the file where the checkers are defined
    checkers_path: PathBuf,
    // path to the file which needs to be checked
    config_path: PathBuf,
    value: toml::Table,
}

impl EntryAbsent {
    fn new(checkers_path: PathBuf, config_path: PathBuf, value: toml::Table) -> Self {
        Self {
            checkers_path,
            config_path,
            value,
        }
    }

    fn value(&self) -> toml::Table {
        self.value.clone()
    }
}

impl Check for EntryAbsent {
    fn style_type(&self) -> String {
        "entry_absent".to_string()
    }

    fn checkers_path(&self) -> &PathBuf {
        &self.checkers_path
    }

    fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    fn get_ist_and_soll(&self) -> Result<IstAndSoll, String> {
        if !self.config_path().exists() {
            return Ok(IstAndSoll {
                ist: "".to_string(),
                soll: "".to_string(),
                action: Action::None,
            });
        }

        let contents = fs::read_to_string(self.config_path());
        if let Err(s) = contents {
            return Err(s.to_string());
        }
        let contents = contents.unwrap();
        let new_contents = checkers::toml::unset(&contents, &self.value).unwrap();
        let action = if contents == new_contents {
            Action::None
        } else {
            Action::SetContents
        };
        Ok(IstAndSoll {
            ist: contents,
            soll: new_contents,
            action,
        })
    }
}
// pub trait YamlEntryPresent {}
// pub trait YamlEntryAbsent {}

// impl YamlEntryPresent for Entry {}
// impl YamlEntryAbsent for Entry {}

// enum EntryType {
//     YamlPresent(dyn YamlEntryPresent),
//     YamlAbsent(dyn YamlEntryAbsent),
//     // TomlPresent(Entry),
//     // TomlAbsent(Entry),
//     // JsonPresent(Entry),
//     // JsonAbsent(Entry),
//     // IniPresent(Entry),
//     // IniAbsent(Entry),
// }
