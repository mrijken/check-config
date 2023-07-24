// mod checkers;
use core::fmt::Debug as DebugTrait;
use std::{
    env,
    fs::{self, OpenOptions},
    path::PathBuf,
};

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

fn read_style(source_path: &PathBuf) -> Vec<Box<dyn Check>> {
    let s = fs::read_to_string(source_path).unwrap();
    let t: toml::Table = toml::from_str(s.as_str()).unwrap();

    let mut styles = vec![];

    for (path, value) in t {
        if path == "nitpick" {
            if value.get("additional_styles").is_none() {
                panic!("additional_styles not found");
            }
            if let Value::Array(includes) = value.get("additional_styles").unwrap() {
                for include_path in includes {
                    styles.extend(read_style(
                        &source_path
                            .parent()
                            .unwrap()
                            .join(include_path.as_str().unwrap()),
                    ))
                }
            }
            continue;
        }
        match value {
            Value::Table(v) => {
                for (check, v) in v {
                    let config_path = env::current_dir().unwrap().join(path.as_str());
                    let check: Option<Box<dyn Check>> = match check.as_str() {
                        "key_absent" => Some(Box::new(KeyAbsent {
                            source_path: source_path.clone(),
                            config_path,
                            table: v.clone(),
                        })),
                        "key_present" => Some(Box::new(KeyPresent {
                            source_path: source_path.clone(),
                            config_path,
                            table: v.clone(),
                        })),
                        "file_absent" => Some(Box::new(FileAbsent {
                            source_path: source_path.clone(),
                            config_path,
                        })),
                        "file_present" => Some(Box::new(FilePresent {
                            source_path: source_path.clone(),
                            config_path,
                        })),
                        "line_present" => Some(Box::new(BlockPresent {
                            source_path: source_path.clone(),
                            config_path,
                            block: v.get("block").unwrap().as_str().unwrap().to_string(),
                        })),
                        "line_absent" => Some(Box::new(BlockAbsent {
                            source_path: source_path.clone(),
                            config_path,
                            block: v.get("block").unwrap().as_str().unwrap().to_string(),
                        })),
                        _ => panic!("unknown check {}", check),
                    };
                    if let Some(check) = check {
                        styles.push(check);
                    }
                }
            }
            Value::Array(v) => {
                dbg!(v);
            }
            _ => {}
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
    table: toml::Value,
}

impl KeyPresent {
    fn new(source_path: PathBuf, config_path: PathBuf, table: toml::Value) -> Self {
        Self {
            source_path,
            config_path,
            table,
        }
    }

    fn table(&self) -> &toml::Value {
        &self.table
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

        let new_contents = merge_toml(&contents, self.table.clone()).unwrap();

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

fn merge_toml(contents: &str, table: toml::Value) -> Result<String, String> {
    let doc: toml::Value = toml::from_str(contents).unwrap();

    let new_doc = serde_toml_merge::merge(doc, table.clone()).unwrap();
    let new_contents = format!("{}", new_doc.as_table().unwrap());
    Ok(new_contents)
}

#[derive(Debug)]
pub struct KeyAbsent {
    // path to the file where the style is defined
    source_path: PathBuf,
    // path the file which needs to be checked
    config_path: PathBuf,
    table: toml::Value,
}

impl KeyAbsent {
    fn new(source_path: PathBuf, config_path: PathBuf, table: toml::Value) -> Self {
        Self {
            source_path,
            config_path,
            table,
        }
    }

    fn table(&self) -> &toml::Value {
        &self.table
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
        let mut doc: toml::Value = toml::from_str(contents.as_str()).unwrap();
        let table = doc.as_table_mut().unwrap();
        for (k, _) in self.table.as_table().unwrap() {
            // todo: nested
            table.remove(k);
        }

        let new_doc = serde_toml_merge::merge(doc, self.table.clone()).unwrap();
        let new_contents = format!("{}", new_doc.as_table().unwrap());
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
