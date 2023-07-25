// mod checkers;
use core::fmt::Debug as DebugTrait;
use std::{
    env,
    fs::{self, OpenOptions},
    path::PathBuf,
};

mod checkers;
use similar::TextDiff;
use toml::Value;

fn main() -> Result<(), String> {
    let styles = read_style(&PathBuf::from(r"styles/python.toml"));
    dbg!(&styles);

    for style in styles {
        match style.check() {
            Ok(ist_and_soll) => match ist_and_soll.action {
                Action::None => {
                    println!(
                        "✅ {} - {} - {}",
                        style.source_path().to_string_lossy(),
                        style.config_path().to_string_lossy(),
                        style.style_type(),
                    );
                }
                Action::RemoveFile => {
                    println!(
                        "❌ {} - {} - {} - file is present",
                        style.source_path().to_string_lossy(),
                        style.config_path().to_string_lossy(),
                        style.style_type(),
                    );
                }
                Action::SetContents => {
                    println!(
                        "❌ {} - {} - {} - diff",
                        style.source_path().to_string_lossy(),
                        style.config_path().to_string_lossy(),
                        style.style_type(),
                    );
                    println!(
                        "{}",
                        TextDiff::from_lines(&ist_and_soll.ist, &ist_and_soll.soll).unified_diff()
                    )
                }
            },
            Err(e) => println!("Error: {}", e),
        }

        // style.fix()?;
    }

    Ok(())
}

struct MetaKeys {
    path_key: String,
    file_key: String,
    check_type_key: String,
}

impl MetaKeys {
    fn new() -> Self {
        Self {
            path_key: "__path__".to_string(),
            file_key: "__file__".to_string(),
            check_type_key: "__check__".to_string(),
        }
    }
    fn from_nitpick_table(table: &toml::Value) -> Self {
        match table.as_table() {
            None => panic!("not a table"),
            Some(table) => {
                let mut meta_keys = Self::new();
                if let Some(new_path_key) = table.get("path_key") {
                    meta_keys.path_key = new_path_key.as_str().unwrap().to_string();
                }
                if let Some(new_file_key) = table.get("file_key") {
                    meta_keys.file_key = new_file_key.as_str().unwrap().to_string();
                }
                if let Some(new_status_key) = table.get("status_key") {
                    meta_keys.check_type_key = new_status_key.as_str().unwrap().to_string();
                }
                meta_keys
            }
        }
    }
}

fn read_style(source_path: &PathBuf) -> Vec<Box<dyn Check>> {
    let s = fs::read_to_string(source_path).unwrap();
    let t: toml::Table = toml::from_str(s.as_str()).unwrap();

    let mut styles = vec![];

    let mut meta_keys = MetaKeys::new();

    for (key, value) in t {
        if key == "nitpick" {
            meta_keys = MetaKeys::from_nitpick_table(&value);
            if let Some(Value::Array(includes)) = value.get("additional_styles") {
                for include_path in includes {
                    styles.extend(read_style(
                        &source_path
                            .parent()
                            .unwrap()
                            .join(include_path.as_str().unwrap()),
                    ))
                }
            }
        }
        match value {
            Value::Table(table) => {
                styles.extend(table2check(
                    &table,
                    &meta_keys,
                    source_path.clone(),
                    vec![key],
                ));
            }
            Value::Array(array) => {
                for element in array {
                    if let Some(table) = element.as_table() {
                        styles.extend(table2check(
                            &table,
                            &meta_keys,
                            source_path.clone(),
                            vec![key.clone()],
                        ));
                    }
                }
            }
            _ => {}
        }
    }
    styles
}

fn table2check(
    table: &toml::Table,
    meta_keys: &MetaKeys,
    source_path: PathBuf,
    key: Vec<String>,
) -> Vec<Box<dyn Check>> {
    let mut styles = vec![];
    dbg!(&table, &key, &source_path, &meta_keys.path_key);
    let source_path = source_path.clone();
    let mut key = key.clone();
    let config_path = env::current_dir()
        .unwrap()
        .join(table.get(&meta_keys.path_key).unwrap().as_str().unwrap());

    match table.get(&meta_keys.check_type_key).unwrap().as_str() {
        Some(check_type) => {
            let check: Option<Box<dyn Check>> = match check_type {
                "file_absent" => Some(Box::new(FileAbsent {
                    source_path,
                    config_path,
                })),
                "file_present" => Some(Box::new(FilePresent {
                    source_path,
                    config_path,
                })),
                "lines_absent" => Some(Box::new(BlockAbsent {
                    source_path,
                    config_path,
                    block: table
                        .get("__lines__")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string(),
                })),
                "lines_present" => Some(Box::new(BlockPresent {
                    source_path,
                    config_path,
                    block: table
                        .get("__lines__")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string(),
                })),
                "key_present" => Some(Box::new(KeyPresent {
                    source_path,
                    config_path,
                    key,
                    value: toml::Table(*table), // without meta keys
                })),
                "key_absent" => Some(Box::new(KeyAbsent {
                    source_path: source_path.clone(),
                    config_path,
                    key,
                })),
                _ => panic!("unknown check {}", check_type),
            };
            if let Some(check) = check {
                styles.push(check);
            }
        }
        None => {
            for (k, v) in table {
                if let Some(v) = v.as_table() {
                    let mut key = key.clone();
                    key.push(k.clone());
                    styles.extend(table2check(&v, meta_keys, source_path.clone(), key))
                }
            }
        }
    }
    styles
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

// impl<'ist, 'soll, 'diff> IstAndSoll {
//     fn get_diff(&mut self) -> TextDiff<'ist, 'soll, 'diff, str> {
//         self.soll = TextDiff::from_lines(&self.ist, &self.soll).unified_diff()
//     }
// }

trait Check: DebugTrait {
    fn style_type(&self) -> String;
    fn get_ist_and_soll(&self) -> Result<IstAndSoll, String>;
    fn source_path(&self) -> &PathBuf;
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
struct BlockPresent {
    // path to the file where the style is defined
    source_path: PathBuf,
    // path the file which needs to be checked
    config_path: PathBuf,
    block: String,
}

impl Check for BlockPresent {
    fn style_type(&self) -> String {
        "block_present".to_string()
    }

    fn source_path(&self) -> &PathBuf {
        &self.source_path
    }

    fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    fn get_ist_and_soll(&self) -> Result<IstAndSoll, String> {
        if !self.config_path().exists() {
            return Ok(IstAndSoll {
                ist: "".to_string(),
                soll: self.block.clone(),
                action: Action::SetContents,
            });
        }
        match fs::read_to_string(self.config_path()) {
            Ok(contents) => {
                if contents.contains(&self.block) {
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
                    new_contents += &self.block.clone();
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
struct BlockAbsent {
    // path to the file where the style is defined
    source_path: PathBuf,
    // path the file which needs to be checked
    config_path: PathBuf,
    block: String,
}
impl Check for BlockAbsent {
    fn style_type(&self) -> String {
        "block_absent".to_string()
    }

    fn source_path(&self) -> &PathBuf {
        &self.source_path
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
    // path to the file where the style is defined
    source_path: PathBuf,
    // path the file which needs to be checked
    config_path: PathBuf,
}

impl Check for FileAbsent {
    fn style_type(&self) -> String {
        "file_absent".to_string()
    }

    fn source_path(&self) -> &PathBuf {
        &self.source_path
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
    // path to the file where the style is defined
    source_path: PathBuf,
    // path the file which needs to be checked
    config_path: PathBuf,
}

impl Check for FilePresent {
    fn style_type(&self) -> String {
        "file_present".to_string()
    }

    fn source_path(&self) -> &PathBuf {
        &self.source_path
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
pub struct KeyPresent {
    // path to the file where the style is defined
    source_path: PathBuf,
    // path the file which needs to be checked
    config_path: PathBuf,
    key: Vec<String>,
    value: toml::Value,
}

impl KeyPresent {
    fn new(
        source_path: PathBuf,
        config_path: PathBuf,
        key: Vec<String>,
        value: toml::Value,
    ) -> Self {
        Self {
            source_path,
            config_path,
            key,
            value,
        }
    }
}

use toml_edit::{value, Document, Item};

impl Check for KeyPresent {
    fn style_type(&self) -> String {
        "key_present".to_string()
    }

    fn source_path(&self) -> &PathBuf {
        &self.source_path
    }

    fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    fn get_ist_and_soll(&self) -> Result<IstAndSoll, String> {
        let contents = fs::read_to_string(self.config_path());
        if let Err(s) = contents {
            return Err(s.to_string());
        }
        let contents = contents.unwrap();

        // let new_contents = checkers::toml::merge(&contents, self.table.clone()).unwrap();
        let new_contents = contents.clone();

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
pub struct KeyAbsent {
    // path to the file where the style is defined
    source_path: PathBuf,
    // path the file which needs to be checked
    config_path: PathBuf,
    key: Vec<String>,
}

impl KeyAbsent {
    fn new(source_path: PathBuf, config_path: PathBuf, key: Vec<String>) -> Self {
        Self {
            source_path,
            config_path,
            key,
        }
    }

    fn key(&self) -> Vec<String> {
        self.key.clone()
    }
}

impl Check for KeyAbsent {
    fn style_type(&self) -> String {
        "key_absent".to_string()
    }

    fn source_path(&self) -> &PathBuf {
        &self.source_path
    }

    fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    fn get_ist_and_soll(&self) -> Result<IstAndSoll, String> {
        let contents = fs::read_to_string(self.config_path());
        if let Err(s) = contents {
            return Err(s.to_string());
        }
        let contents = contents.unwrap();
        // let new_contents = checkers::toml::remove_key(&contents, self.table.clone()).unwrap();
        let new_contents = contents.clone();
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
