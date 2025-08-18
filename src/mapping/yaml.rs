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
        return Ok(Box::new(serde_yaml_ng::Mapping::new()));
    }
    let doc: serde_yaml_ng::Value =
        serde_yaml_ng::from_str(doc).map_err(|e| CheckError::InvalidFileFormat(e.to_string()))?;
    Ok(Box::new(
        doc.as_mapping()
            .ok_or(CheckError::InvalidFileFormat("No object".to_string()))?
            .clone(),
    ))
}

impl Mapping for serde_yaml_ng::Mapping {
    fn to_string(&self) -> Result<String, CheckError> {
        if self.is_empty() {
            return Ok("".to_string());
        }
        Ok(serde_yaml_ng::to_string(&self).unwrap())
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
                serde_yaml_ng::value::Value::String(key.to_string()),
                serde_yaml_ng::value::Value::Mapping(serde_yaml_ng::Mapping::new()),
            );
        }
        let value = self.get_mut(key).unwrap();
        if !value.is_mapping() {
            Err(MappingError::WrongType(format!("{key} is not a mapping")))
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
                serde_yaml_ng::value::Value::String(key.to_string()),
                serde_yaml_ng::value::Value::Sequence(serde_yaml_ng::Sequence::new()),
            );
        }
        let value = self.get_mut(key).unwrap();
        if !value.is_sequence() {
            Err(MappingError::WrongType(format!("{key} is not an array")))
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
            Err(MappingError::WrongType(format!("{key} is not a string")))
        } else {
            Ok(value.as_str().unwrap().to_string())
        }
    }
    fn insert(&mut self, key: &toml_edit::Key, value: &toml_edit::Item) {
        self.insert(
            serde_yaml_ng::value::Value::String(key.to_string()),
            serde_yaml_ng::Value::from_toml_value(value),
        );
    }
    fn remove(&mut self, key: &str) {
        if self.contains_key(key) {
            self.remove(key);
        }
    }
}

impl Array for serde_yaml_ng::value::Sequence {
    fn insert_when_not_present(&mut self, value: &toml_edit::Item) {
        let value = serde_yaml_ng::value::Value::from_toml_value(value);
        if !self.contains(&value) {
            self.push(value);
        }
    }

    fn remove(&mut self, value: &toml_edit::Item) {
        let value = serde_yaml_ng::value::Value::from_toml_value(value);
        let array = self;
        for (idx, array_item) in array.iter().enumerate() {
            if *array_item == value {
                array.remove(idx);
                return;
            }
        }
    }

    fn contains_item(&self, value: &toml_edit::Item) -> bool {
        let value = serde_yaml_ng::value::Value::from_toml_value(value);
        self.contains(&value)
    }
}

impl Value for serde_yaml_ng::value::Value {
    fn from_toml_value(value: &toml_edit::Item) -> serde_yaml_ng::value::Value {
        match value {
            toml_edit::Item::Value(toml_edit::Value::String(v)) => {
                serde_yaml_ng::value::Value::String(v.value().to_owned())
            }
            toml_edit::Item::Value(toml_edit::Value::Integer(v)) => {
                serde_yaml_ng::value::Value::Number(serde_yaml_ng::Number::from(
                    v.value().to_owned(),
                ))
            }
            toml_edit::Item::Value(toml_edit::Value::Float(v)) => {
                serde_yaml_ng::value::Value::Number(serde_yaml_ng::Number::from(
                    v.value().to_owned(),
                ))
            }
            toml_edit::Item::Value(toml_edit::Value::Boolean(v)) => {
                serde_yaml_ng::value::Value::Bool(v.value().to_owned())
            }
            toml_edit::Item::Value(toml_edit::Value::Datetime(v)) => {
                serde_yaml_ng::value::Value::String(v.to_string())
            }
            toml_edit::Item::Value(toml_edit::Value::Array(v)) => {
                let mut a = vec![];
                for v_item in v {
                    a.push(serde_yaml_ng::value::Value::from_toml_value(
                        &toml_edit::Item::Value(v_item.to_owned()),
                    ))
                }
                serde_yaml_ng::value::Value::Sequence(a)
            }
            toml_edit::Item::Table(v) => {
                let mut a: serde_yaml_ng::value::Mapping = serde_yaml_ng::value::Mapping::new();
                for (k, v_item) in v {
                    a.insert(
                        serde_yaml_ng::value::Value::String(k.to_owned()),
                        serde_yaml_ng::value::Value::from_toml_value(v_item),
                    );
                }

                serde_yaml_ng::value::Value::Mapping(a)
            }
            toml_edit::Item::Value(toml_edit::Value::InlineTable(v)) => {
                let mut a: serde_yaml_ng::value::Mapping = serde_yaml_ng::value::Mapping::new();
                for (k, v_item) in v {
                    a.insert(
                        serde_yaml_ng::value::Value::String(k.to_owned()),
                        serde_yaml_ng::value::Value::from_toml_value(&toml_edit::Item::Value(
                            v_item.to_owned(),
                        )),
                    );
                }

                serde_yaml_ng::value::Value::Mapping(a)
            }
            toml_edit::Item::ArrayOfTables(v) => {
                let mut a = vec![];
                for v_item in v {
                    a.push(serde_yaml_ng::value::Value::from_toml_value(
                        &toml_edit::Item::Table(v_item.to_owned()),
                    ))
                }
                serde_yaml_ng::value::Value::Sequence(a)
            }
            _ => serde_yaml_ng::value::Value::Null,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use super::super::generic::tests::get_test_table;
    use super::super::generic::tests::test_mapping;

    #[test]
    fn test_access_map() {
        let table = get_test_table();
        let binding = serde_yaml_ng::Value::from_toml_value(&table);
        let mut mapping_to_check = binding.as_mapping().unwrap().to_owned();

        test_mapping(Box::new(mapping_to_check.clone()));

        assert_eq!(
            mapping_to_check
                .get_mapping("dict", false)
                .expect("")
                .to_string()
                .unwrap(),
            "str: string\nint: 1\nfloat: 1.1\nbool: true\narray:\n- 1\n- 2\n".to_string()
        );
    }

    #[test]
    fn test_from_toml_value() {
        let table = get_test_table();

        let yaml_table = serde_yaml_ng::Value::from_toml_value(&table);

        assert_eq!(serde_yaml_ng
            ::to_string(&yaml_table).unwrap(), "str: string\nint: 1\nfloat: 1.1\nbool: true\narray:\n- 1\n- 2\ndict:\n  str: string\n  int: 1\n  float: 1.1\n  bool: true\n  array:\n  - 1\n  - 2\n"

        );
    }
}
