use std::{fs, path::PathBuf};

use serde::Serialize;

use crate::checkers::base::CheckError;

use super::generic::{Array, Mapping, MappingError, Value};

pub(crate) fn from_path(path: PathBuf) -> Result<Box<dyn Mapping>, CheckError> {
    let file_contents = fs::read_to_string(path)?;
    from_string(&file_contents)
}

pub(crate) fn from_string(doc: &str) -> Result<Box<dyn Mapping>, CheckError> {
    if doc.trim().is_empty() {
        return Ok(Box::new(serde_json::Map::new()));
    }
    let doc: serde_json::Value =
        serde_json::from_str(doc).map_err(|e| CheckError::InvalidFileFormat(e.to_string()))?;
    let doc = doc
        .as_object()
        .ok_or(CheckError::InvalidFileFormat("No object".to_string()))?;
    Ok(Box::new(doc.clone()))
}

impl Mapping for serde_json::Map<String, serde_json::Value> {
    fn to_string(&self) -> Result<String, CheckError> {
        let buf = Vec::new();

        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
        let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
        self.serialize(&mut ser).unwrap();
        Ok(String::from_utf8(ser.into_inner()).unwrap())
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
                key.to_string(),
                serde_json::Value::Object(serde_json::Map::new()),
            );
        }
        let value = self.get_mut(key).unwrap();
        if !value.is_object() {
            Err(MappingError::WrongType(format!("{} is not a mapping", key)))
        } else {
            Ok(value.as_object_mut().unwrap())
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
            self.insert(key.to_string(), serde_json::Value::Array(vec![]));
        }
        let value = self.get_mut(key).unwrap();
        if !value.is_array() {
            Err(MappingError::WrongType(format!("{} is not an array", key)))
        } else {
            Ok(value)
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
        self.insert(key.to_string(), serde_json::Value::from_toml_value(value));
    }
    fn remove(&mut self, key: &str) {
        if self.contains_key(key) {
            self.remove(key);
        }
    }
}

impl Array for serde_json::Value {
    fn insert_when_not_present(&mut self, value: &toml::Value) {
        let value = serde_json::Value::from_toml_value(value);
        if !self.as_array().unwrap().contains(&value) {
            self.as_array_mut().unwrap().push(value);
        }
    }

    fn remove(&mut self, value: &toml::Value) {
        let value = serde_json::Value::from_toml_value(value);
        let array = self.as_array_mut().unwrap();
        for (idx, array_item) in array.iter().enumerate() {
            if *array_item == value {
                array.remove(idx);
                return;
            }
        }
    }

    fn contains_item(&self, value: &toml::Value) -> bool {
        let value = serde_json::Value::from_toml_value(value);
        self.as_array().unwrap().contains(&value)
    }
}

impl Value for serde_json::Value {
    fn from_toml_value(value: &toml::Value) -> serde_json::Value {
        match value {
            toml::Value::String(v) => serde_json::Value::String(v.clone()),
            toml::Value::Integer(v) => serde_json::Value::Number(serde_json::Number::from(*v)),
            toml::Value::Float(v) => {
                serde_json::Value::Number(serde_json::Number::from_f64(*v).unwrap())
            }
            toml::Value::Boolean(v) => serde_json::Value::Bool(*v),
            toml::Value::Datetime(v) => serde_json::Value::String(v.to_string()),
            toml::Value::Array(v) => {
                let mut a = vec![];
                for v_item in v {
                    a.push(serde_json::Value::from_toml_value(v_item))
                }
                serde_json::Value::Array(a)
            }
            toml::Value::Table(v) => {
                let mut a: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
                for (k, v_item) in v {
                    a.insert(k.clone(), serde_json::Value::from_toml_value(v_item));
                }

                serde_json::Value::Object(a)
            }
        }
    }
}
