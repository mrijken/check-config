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

    fn insert(&mut self, key: &toml_edit::Key, value: &toml_edit::Item) {
        self.as_table_mut()
            .insert_formatted(key, toml_edit::Item::from_toml_value(value));
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
            self.insert_formatted(
                &toml_edit::Key::new(key),
                toml_edit::Item::Table(toml_edit::Table::new()),
            );
        }
        let value = self.get_mut(key).unwrap();
        if !value.is_table_like() {
            Err(MappingError::WrongType(format!("{key} is not a mapping")))
        } else if value.is_table() {
            let value = value.as_table_mut().unwrap();
            Ok(value)
        } else {
            let value = value.as_inline_table_mut().unwrap();
            Ok(value)
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
            self.insert_formatted(
                &toml_edit::Key::new(key),
                toml_edit::Item::Value(toml_edit::Value::Array(toml_edit::Array::new())),
            );
        }
        let value = self.get_mut(key).unwrap();
        if !value.is_array() {
            Err(MappingError::WrongType(format!("`{key}` is not an array")))
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
            Err(MappingError::WrongType(format!("{key} is not a string")))
        } else {
            Ok(value.as_str().unwrap().to_string())
        }
    }
    fn insert(&mut self, key: &toml_edit::Key, value: &toml_edit::Item) {
        self.insert_formatted(key, toml_edit::Item::from_toml_value(value));
    }
    fn remove(&mut self, key: &str) {
        if self.contains_key(key) {
            self.remove(key);
        }
    }
}

impl Mapping for toml_edit::InlineTable {
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
            self.insert_formatted(
                &toml_edit::Key::new(key),
                toml_edit::Value::InlineTable(toml_edit::InlineTable::new()),
            );
        }
        let value = self.get_mut(key).unwrap();
        if !value.is_inline_table() {
            Err(MappingError::WrongType(format!("{key} is not a mapping")))
        } else {
            let value = value.as_inline_table_mut().unwrap();
            Ok(value)
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
            self.insert_formatted(
                &toml_edit::Key::new(key),
                toml_edit::Value::Array(toml_edit::Array::new()),
            );
        }
        let value = self.get_mut(key).unwrap();
        if !value.is_array() {
            Err(MappingError::WrongType(format!("`{key}` is not an array")))
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
            Err(MappingError::WrongType(format!("{key} is not a string")))
        } else {
            Ok(value.as_str().unwrap().to_string())
        }
    }
    fn insert(&mut self, key: &toml_edit::Key, value: &toml_edit::Item) {
        self.insert_formatted(
            key,
            toml_edit::Item::from_toml_value(value)
                .as_value()
                .expect("expect value, no table")
                .to_owned(),
        );
    }
    fn remove(&mut self, key: &str) {
        if self.contains_key(key) {
            self.remove(key);
        }
    }
}

impl Array for toml_edit::Array {
    fn insert_when_not_present(&mut self, value: &toml_edit::Item) {
        let value = toml_edit::Item::from_toml_value(value);

        if toml_array_index_equal_without_formatting(self, &value).is_none() {
            self.push_formatted(value.as_value().expect("expect value, no table").to_owned());
        }
    }

    fn remove(&mut self, value: &toml_edit::Item) {
        let value = toml_edit::Item::from_toml_value(value);
        if let Some(idx) = toml_array_index_equal_without_formatting(self, &value) {
            self.remove(idx);
        }
    }

    fn contains_item(&self, value: &toml_edit::Item) -> bool {
        let value = toml_edit::Item::from_toml_value(value);
        toml_array_index_equal_without_formatting(self, &value).is_some()
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

fn toml_array_index_equal_without_formatting(
    array: &toml_edit::Array,
    item: &toml_edit::Item,
) -> Option<usize> {
    for (idx, array_item) in array.iter().enumerate() {
        if item_value_equals(array_item, item.as_value().unwrap()) {
            return Some(idx);
        }
    }
    None
}

impl Value for toml_edit::Item {
    fn from_toml_value(value: &toml_edit::Item) -> Self {
        value.clone()
    }
}
#[cfg(test)]
mod tests {

    use super::super::generic::tests::get_test_table;
    use super::super::generic::tests::test_mapping;
    use super::*;

    #[test]
    fn test_access_map() {
        let table = get_test_table().into_table().unwrap();

        test_mapping(Box::new(table.clone()));
    }

    #[test]
    fn test_from_toml_value() {
        let table = get_test_table();

        let toml_table = toml_edit::Item::from_toml_value(&table);

        assert_eq!(
            toml_table.to_string(),
            r#"str = "string"
# comment for int
int = 1
float = 1.1
bool = true
array = [
  # comment for item 1
  1,
  # comment for item 2
  2
]
dict = { str = "string", int = 1, float = 1.1, bool = true, array = [1, 2] }
"#
        );
    }

    #[test]
    fn test_insert_and_remove_with_comments() {
        let old_doc = "
[table]
# prefix comment Key \"keep_int\" 
keep_int = 2  # suffix comment Value 2
# prefix comment Key \"remove_int\"
remove_int = 3  # suffix comment Value 3
keep_array = [ # prefix comment Value \"keep_item_1\"
    \"keep_item_1\", # prefix comment remove_item_2
    \"remove_item_2\"  # suffix comment remove_item_2
    ,
    # prefix comment item keep_item_3
    \"keep_item_3\" # suffix comment item keep_item_3
] # suffix comment Array \"keep_array\"
        "
        .parse::<toml_edit::DocumentMut>()
        .unwrap();

        let items_to_add = "
[table]
# prefix comment Key \"add_int\" 
add_int = 2  # suffix comment Value 2
keep_array = [ # prefix comment \"add_item_4\"
    \"add_item_4\" # suffix comment \"add_item_4\"
]
"
        .parse::<toml_edit::DocumentMut>()
        .unwrap()["table"]
            .as_table()
            .unwrap()
            .clone();

        let items_to_remove = "
[table]
remove_int = 2
keep_array = [
    \"remove_item_2\"
]
"
        .parse::<toml_edit::DocumentMut>()
        .unwrap()["table"]
            .as_table()
            .unwrap()
            .clone();

        let mut new_doc = old_doc.clone();

        let (key, value) = items_to_add.get_key_value("add_int").unwrap();
        Mapping::insert(new_doc["table"].as_table_mut().unwrap(), key, value);

        let item = items_to_add["keep_array"]
            .as_array()
            .unwrap()
            .get(0)
            .unwrap();
        Array::insert_when_not_present(
            new_doc["table"]["keep_array"].as_array_mut().unwrap(),
            &toml_edit::Item::Value(item.clone()),
        );

        let (key, _value) = items_to_remove.get_key_value("remove_int").unwrap();
        Mapping::remove(new_doc["table"].as_table_mut().unwrap(), key);

        let item = items_to_remove["keep_array"]
            .as_array()
            .unwrap()
            .get(0)
            .unwrap();
        Array::remove(
            new_doc["table"].as_table_mut().unwrap()["keep_array"]
                .as_array_mut()
                .unwrap(),
            &toml_edit::Item::Value(item.clone()),
        );

        new_doc.fmt();

        assert_eq!(
            Mapping::to_string(&new_doc).unwrap(),
            "\n[table]\n# prefix comment Key \"keep_int\" \nkeep_int = 2  # suffix comment Value 2\nkeep_array = [ # prefix comment Value \"keep_item_1\"\n    \"keep_item_1\",\n    # prefix comment item keep_item_3\n    \"keep_item_3\" # suffix comment item keep_item_3\n, # prefix comment \"add_item_4\"\n    \"add_item_4\" # suffix comment \"add_item_4\"\n] # suffix comment Array \"keep_array\"\n# prefix comment Key \"add_int\" \nadd_int = 2  # suffix comment Value 2\n        "
        );
    }
}
