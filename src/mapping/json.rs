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
        if self.is_empty() {
            return Ok("".to_string());
        }
        let buf = Vec::new();

        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
        let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
        self.serialize(&mut ser).unwrap();
        Ok(String::from_utf8(ser.into_inner()).unwrap() + "\n")
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
            Err(MappingError::WrongType(format!("{key} is not a mapping")))
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
            Err(MappingError::WrongType(format!("{key} is not an array")))
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
            Err(MappingError::WrongType(format!("{key} is not a string")))
        } else {
            Ok(value.as_str().unwrap().to_string())
        }
    }
    fn insert(&mut self, key: &toml_edit::Key, value: &toml_edit::Item) {
        self.insert(key.to_string(), serde_json::Value::from_toml_value(value));
    }
    fn remove(&mut self, key: &str) {
        if self.contains_key(key) {
            self.remove(key);
        }
    }
}

impl Array for serde_json::Value {
    fn insert_when_not_present(&mut self, value: &toml_edit::Item) {
        let value = serde_json::Value::from_toml_value(value);
        if !self.as_array().unwrap().contains(&value) {
            self.as_array_mut().unwrap().push(value);
        }
    }

    fn remove(&mut self, value: &toml_edit::Item) {
        let value = serde_json::Value::from_toml_value(value);
        let array = self.as_array_mut().unwrap();
        for (idx, array_item) in array.iter().enumerate() {
            if *array_item == value {
                array.remove(idx);
                return;
            }
        }
    }

    fn contains_item(&self, value: &toml_edit::Item) -> bool {
        let value = serde_json::Value::from_toml_value(value);
        self.as_array().unwrap().contains(&value)
    }
}

impl Value for serde_json::Value {
    fn from_toml_value(value: &toml_edit::Item) -> serde_json::Value {
        match value {
            toml_edit::Item::Value(toml_edit::Value::String(v)) => {
                serde_json::Value::String(v.value().to_owned())
            }
            toml_edit::Item::Value(toml_edit::Value::Integer(v)) => {
                serde_json::Value::Number(serde_json::Number::from(*v.value()))
            }
            toml_edit::Item::Value(toml_edit::Value::Float(v)) => {
                serde_json::Value::Number(serde_json::Number::from_f64(*v.value()).unwrap())
            }
            toml_edit::Item::Value(toml_edit::Value::Boolean(v)) => {
                serde_json::Value::Bool(*v.value())
            }
            toml_edit::Item::Value(toml_edit::Value::Datetime(v)) => {
                serde_json::Value::String(v.value().to_string())
            }
            toml_edit::Item::Value(toml_edit::Value::Array(v)) => {
                let mut a = vec![];
                for v_item in v {
                    a.push(serde_json::Value::from_toml_value(&toml_edit::Item::Value(
                        v_item.to_owned(),
                    )))
                }
                serde_json::Value::Array(a)
            }
            toml_edit::Item::Table(v) => {
                let mut a: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
                for (k, v_item) in v {
                    a.insert(k.to_string(), serde_json::Value::from_toml_value(v_item));
                }

                serde_json::Value::Object(a)
            }
            toml_edit::Item::Value(toml_edit::Value::InlineTable(v)) => {
                let mut a: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
                for (k, v_item) in v {
                    a.insert(
                        k.to_string(),
                        serde_json::Value::from_toml_value(&toml_edit::Item::Value(
                            v_item.to_owned(),
                        )),
                    );
                }

                serde_json::Value::Object(a)
            }
            toml_edit::Item::ArrayOfTables(v) => {
                let mut a = vec![];
                for v_item in v {
                    a.push(serde_json::Value::from_toml_value(&toml_edit::Item::Table(
                        v_item.to_owned(),
                    )))
                }
                serde_json::Value::Array(a)
            }
            _ => serde_json::Value::Null,
        }
    }
}

#[cfg(test)]
mod tests {

    use serde_json::json;

    use super::super::generic::tests::get_test_table;
    use super::super::generic::tests::test_mapping;
    use super::*;

    #[test]
    fn test_access_map() {
        let table = get_test_table();
        let binding = serde_json::Value::from_toml_value(&table);
        let mut mapping_to_check = binding.as_object().unwrap().to_owned();

        test_mapping(Box::new(mapping_to_check.clone()));

        assert_eq!(
            mapping_to_check
                .get_mapping("dict", false)
                .expect("")
                .to_string()
                .unwrap(),
            "{\n    \"array\": [\n        1,\n        2\n    ],\n    \"bool\": true,\n    \"float\": 1.1,\n    \"int\": 1,\n    \"str\": \"string\"\n}\n".to_string()
        );
    }

    #[test]
    fn test_from_toml_value() {
        let table = get_test_table();

        let json_table = serde_json::Value::from_toml_value(&table);

        assert_eq!(
            json_table,
            json!({ "str": "string", "int": 1, "float": 1.1, "bool": true, "array": [1, 2], "dict": {"str": "string", "int": 1, "float": 1.1, "bool": true, "array": [1, 2],


            }})
        );
    }
}
