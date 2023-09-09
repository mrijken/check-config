use std::{fs, path::PathBuf};

use crate::checkers::base::CheckError;

use super::generic::{Array, Mapping, MappingError, Value};

pub(crate) fn from_path(path: PathBuf) -> Result<Box<dyn Mapping>, CheckError> {
    let file_contents = fs::read_to_string(path)?;
    from_string(&file_contents)
}

pub(crate) fn from_string(
    doc: &str,
) -> Result<Box<dyn Mapping>, crate::checkers::base::CheckError> {
    if doc.trim().is_empty() {
        return Ok(Box::new(serde_yaml::Mapping::new()));
    }
    let doc: serde_yaml::Value =
        serde_yaml::from_str(doc).map_err(|e| CheckError::InvalidFileFormat(e.to_string()))?;
    Ok(Box::new(
        doc.as_mapping()
            .ok_or(CheckError::InvalidFileFormat("No object".to_string()))?
            .clone(),
    ))
}

impl Mapping for serde_yaml::Mapping {
    fn to_string(&self) -> Result<String, CheckError> {
        Ok(serde_yaml::to_string(&self).unwrap())
    }
    fn contains_key(&self, key: &str) -> bool {
        self.contains_key(key)
    }
    fn get_mapping(
        &mut self,
        key: &str,
        create_missing: bool,
    ) -> Result<&mut dyn Mapping, MappingError> {
        if !self.contains_key(key) {
            if !create_missing {
                return Err(MappingError::MissingKey(key.to_string()));
            }
            self.insert(
                serde_yaml::value::Value::String(key.to_string()),
                serde_yaml::value::Value::Mapping(serde_yaml::Mapping::new()),
            );
        }
        let value = self.get_mut(key).unwrap();
        if !value.is_mapping() {
            Err(MappingError::WrongType(format!("{} is not a mapping", key)))
        } else {
            Ok(value.as_mapping_mut().unwrap())
        }
    }
    fn get_array(
        &mut self,
        key: &str,
        create_missing: bool,
    ) -> Result<&mut dyn Array, MappingError> {
        if !self.contains_key(key) {
            if !create_missing {
                return Err(MappingError::MissingKey(key.to_string()));
            }
            self.insert(
                serde_yaml::value::Value::String(key.to_string()),
                serde_yaml::value::Value::Sequence(serde_yaml::Sequence::new()),
            );
        }
        let value = self.get_mut(key).unwrap();
        if !value.is_sequence() {
            Err(MappingError::WrongType(format!("{} is not an array", key)))
        } else {
            Ok(value.as_sequence_mut().unwrap())
        }
    }
    fn get_string(&self, key: &str) -> Result<String, MappingError> {
        if !self.contains_key(key) {
            return Err(MappingError::MissingKey(key.to_string()));
        }
        let value = self.get(key).unwrap();
        if !value.is_string() {
            Err(MappingError::WrongType(format!("{} is not a string", key)))
        } else {
            Ok(value.as_str().unwrap().to_string())
        }
    }
    fn insert(&mut self, key: &str, value: &toml::Value) {
        self.insert(
            serde_yaml::value::Value::String(key.to_string()),
            serde_yaml::Value::from_toml_value(value),
        );
    }
    fn remove(&mut self, key: &str) {
        if self.contains_key(key) {
            self.remove(key);
        }
    }
}

impl Array for serde_yaml::value::Sequence {
    fn insert_when_not_present(&mut self, value: &toml::Value) {
        let value = serde_yaml::value::Value::from_toml_value(value);
        if !self.contains(&value) {
            self.push(value);
        }
    }

    fn remove(&mut self, value: &toml::Value) {
        let value = serde_yaml::value::Value::from_toml_value(value);
        let array = self;
        for (idx, array_item) in array.iter().enumerate() {
            if *array_item == value {
                array.remove(idx);
                return;
            }
        }
    }

    fn contains_item(&self, value: &toml::Value) -> bool {
        let value = serde_yaml::value::Value::from_toml_value(value);
        self.contains(&value)
    }
}

impl Value for serde_yaml::value::Value {
    fn from_toml_value(value: &toml::Value) -> serde_yaml::value::Value {
        match value {
            toml::Value::String(v) => serde_yaml::value::Value::String(v.clone()),
            toml::Value::Integer(v) => {
                serde_yaml::value::Value::Number(serde_yaml::Number::from(*v))
            }
            toml::Value::Float(v) => serde_yaml::value::Value::Number(serde_yaml::Number::from(*v)),
            toml::Value::Boolean(v) => serde_yaml::value::Value::Bool(*v),
            toml::Value::Datetime(v) => serde_yaml::value::Value::String(v.to_string()),
            toml::Value::Array(v) => {
                let mut a = vec![];
                for v_item in v {
                    a.push(serde_yaml::value::Value::from_toml_value(v_item))
                }
                serde_yaml::value::Value::Sequence(a)
            }
            toml::Value::Table(v) => {
                let mut a: serde_yaml::value::Mapping = serde_yaml::value::Mapping::new();
                for (k, v_item) in v {
                    a.insert(
                        serde_yaml::value::Value::String(k.clone()),
                        serde_yaml::value::Value::from_toml_value(v_item),
                    );
                }

                serde_yaml::value::Value::Mapping(a)
            }
        }
    }
}
