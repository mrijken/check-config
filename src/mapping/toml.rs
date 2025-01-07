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
    let doc = doc
        .parse::<toml_edit::DocumentMut>()
        .map_err(|e| CheckError::InvalidFileFormat(e.to_string()))?;
    Ok(Box::new(doc.clone()))
}

impl Mapping for toml_edit::DocumentMut {
    fn to_string(&self) -> Result<String, CheckError> {
        Ok(std::string::ToString::to_string(&self))
    }

    fn get_mapping(
        &mut self,
        key: &str,
        create_missing: bool,
    ) -> Result<&mut dyn Mapping, MappingError> {
        self.as_table_mut().get_mapping(key, create_missing)
    }

    fn contains_key(&self, key: &str) -> bool {
        self.as_table().contains_key(key)
    }

    fn get_array(
        &mut self,
        key: &str,
        create_missing: bool,
    ) -> Result<&mut dyn Array, MappingError> {
        self.as_table_mut().get_array(key, create_missing)
    }

    fn get_string(&self, key: &str) -> Result<String, MappingError> {
        self.as_table().get_string(key)
    }

    fn insert(&mut self, key: &str, value: &toml::Value) {
        self.as_table_mut().insert(
            key,
            toml_edit::Item::Value(toml_edit::Value::from_toml_value(value)),
        );
    }
    fn remove(&mut self, key: &str) {
        if self.as_table().contains_key(key) {
            self.as_table_mut().remove(key);
        }
    }
}

impl Mapping for toml_edit::Table {
    fn to_string(&self) -> Result<String, CheckError> {
        log::error!("not implemented, call to_string on Document instead");
        std::process::exit(1);
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
            self.insert(key, toml_edit::Item::Table(toml_edit::Table::new()));
        }
        let value = self.get_mut(key).unwrap();
        if !value.is_table_like() {
            Err(MappingError::WrongType(format!("{} is not a mapping", key)))
        } else {
            Ok(value.as_table_mut().unwrap())
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
                key,
                toml_edit::Item::Value(toml_edit::Value::Array(toml_edit::Array::new())),
            );
        }
        let value = self.get_mut(key).unwrap();
        if !value.is_array() {
            Err(MappingError::WrongType(format!(
                "`{}` is not an array",
                key
            )))
        } else {
            Ok(value.as_array_mut().unwrap())
        }
    }
    fn get_string(&self, key: &str) -> Result<String, MappingError> {
        if !self.contains_key(key) {
            return Err(MappingError::MissingKey(key.to_string()));
        }
        let value = self.get(key).unwrap();
        if !value.is_str() {
            Err(MappingError::WrongType(format!("{} is not a string", key)))
        } else {
            Ok(value.as_str().unwrap().to_string())
        }
    }
    fn insert(&mut self, key: &str, value: &toml::Value) {
        self.insert(
            key,
            toml_edit::Item::Value(toml_edit::Value::from_toml_value(value)),
        );
    }
    fn remove(&mut self, key: &str) {
        if self.contains_key(key) {
            self.remove(key);
        }
    }
}

impl Array for toml_edit::Array {
    fn insert_when_not_present(&mut self, value: &toml::Value) {
        let value = toml_edit::Value::from_toml_value(value);
        if toml_edit_array_index(self, &value).is_none() {
            self.push(value);
        }
    }

    fn remove(&mut self, value: &toml::Value) {
        let value = toml_edit::Value::from_toml_value(value);
        if let Some(idx) = toml_edit_array_index(self, &value) {
            self.remove(idx);
            self.fmt(); // make sure spaces around removed item are removed also
        }
    }

    fn contains_item(&self, value: &toml::Value) -> bool {
        let value = toml_edit::Value::from_toml_value(value);
        toml_edit_array_index(self, &value).is_some()
    }
}

fn item_value_equals(item: &toml_edit::Value, value: &toml_edit::Value) -> bool {
    match (item, value) {
        (toml_edit::Value::String(item), toml_edit::Value::String(value)) => {
            item.value() == value.value()
        }
        (toml_edit::Value::Integer(item), toml_edit::Value::Integer(value)) => {
            item.value() == value.value()
        }
        (toml_edit::Value::Float(item), toml_edit::Value::Float(value)) => {
            item.value() == value.value()
        }
        (toml_edit::Value::Boolean(item), toml_edit::Value::Boolean(value)) => {
            item.value() == value.value()
        }
        (toml_edit::Value::Array(items), toml_edit::Value::Array(values)) => {
            if items.len() != values.len() {
                return false;
            }
            for (i, j) in items.iter().zip(values) {
                if !item_value_equals(i, j) {
                    return false;
                }
            }
            true
        }
        (toml_edit::Value::InlineTable(items), toml_edit::Value::InlineTable(values)) => {
            if items.len() != values.len() {
                return false;
            }
            for (i, j) in items.iter().zip(values) {
                if i.0 != j.0 || !item_value_equals(i.1, j.1) {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}

fn toml_edit_array_index(array: &toml_edit::Array, item: &toml_edit::Value) -> Option<usize> {
    for (idx, array_item) in array.iter().enumerate() {
        if item_value_equals(array_item, item) {
            return Some(idx);
        }
    }
    None
}

impl Value for toml_edit::Value {
    fn from_toml_value(value: &toml::Value) -> Self {
        match value {
            toml::Value::String(v) => {
                toml_edit::Value::String(toml_edit::Formatted::new(v.clone()))
            }
            toml::Value::Integer(v) => toml_edit::Value::Integer(toml_edit::Formatted::new(*v)),
            toml::Value::Float(v) => toml_edit::Value::Float(toml_edit::Formatted::new(*v)),
            toml::Value::Boolean(v) => toml_edit::Value::Boolean(toml_edit::Formatted::new(*v)),
            toml::Value::Datetime(v) => toml_edit::Value::Datetime(toml_edit::Formatted::new(*v)),
            toml::Value::Array(v) => {
                let mut a = toml_edit::Array::new();
                for v_item in v {
                    a.push_formatted(toml_edit::Value::from_toml_value(v_item))
                }
                toml_edit::Value::Array(a)
            }
            toml::Value::Table(v) => {
                let mut a = toml_edit::InlineTable::new();
                for (k, v_item) in v {
                    a.insert(k, toml_edit::Value::from_toml_value(v_item));
                }

                toml_edit::Value::InlineTable(a)
            }
        }
    }
}
