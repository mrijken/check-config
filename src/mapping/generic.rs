use derive_more::derive::Display;

use crate::checkers::base::CheckError;

#[derive(Debug, Display)]
pub(crate) enum MappingError {
    MissingKey(String),
    WrongType(String),
}

pub(crate) trait Mapping: Send + Sync {
    fn to_string(&self) -> Result<String, CheckError>;

    fn get_mapping(
        &mut self,
        key: &str,
        create_missing: bool,
    ) -> Result<&mut dyn Mapping, MappingError>;
    fn contains_key(&self, key: &str) -> bool;
    fn get_array(
        &mut self,
        key: &str,
        create_missing: bool,
    ) -> Result<&mut dyn Array, MappingError>;
    fn get_string(&self, key: &str) -> Result<String, MappingError>;
    fn insert(&mut self, key: &str, value: &toml_edit::Item);
    fn remove(&mut self, key: &str);
}

pub(crate) trait Array {
    fn insert_when_not_present(&mut self, value: &toml_edit::Item);

    fn remove(&mut self, value: &toml_edit::Item);

    fn contains_item(&self, value: &toml_edit::Item) -> bool;
}
pub(crate) trait Value {
    fn from_toml_value(value: &toml_edit::Item) -> Self
    where
        Self: Sized;
}

#[cfg(test)]
pub(crate) mod tests {

    use super::Mapping;

    pub(crate) fn get_test_table() -> toml_edit::Item {
        let mut table = toml_edit::Table::new();
        table.insert(
            "str",
            toml_edit::Item::Value(toml_edit::Value::from("string")),
        );
        table.insert("int", toml_edit::Item::Value(toml_edit::Value::from(1)));
        table.insert("float", toml_edit::Item::Value(toml_edit::Value::from(1.1)));
        table.insert("bool", toml_edit::Item::Value(toml_edit::Value::from(true)));
        table.insert(
            "array",
            toml_edit::Item::Value(toml_edit::Value::Array(toml_edit::Array::from_iter(vec![
                1,
            ]))),
        );
        let mut nested_table = toml_edit::InlineTable::new();
        nested_table.insert("str", toml_edit::Value::from("string"));
        nested_table.insert("int", toml_edit::Value::from(1));
        nested_table.insert("float", toml_edit::Value::from(1.1));
        nested_table.insert("bool", toml_edit::Value::from(true));
        nested_table.insert(
            "array",
            toml_edit::Value::Array(toml_edit::Array::from_iter(vec![1])),
        );
        table.insert("dict", nested_table.into());

        table.into()
    }

    pub(crate) fn test_mapping(mut mapping_to_check: Box<dyn Mapping>) {
        assert!(mapping_to_check
            .get_array("array", false)
            .expect("")
            .contains_item(&toml_edit::Item::from(toml_edit::Value::from(1))));

        assert_eq!(
            mapping_to_check.get_string("str").expect(""),
            "string".to_string()
        );
        assert!(mapping_to_check.get_string("int").is_err(),);
        assert!(mapping_to_check.get_string("absent").is_err(),);
        assert!(mapping_to_check.get_array("absent", false).is_err(),);
        assert_eq!(
            mapping_to_check
                .get_mapping("dict", false)
                .expect("")
                .get_string("str")
                .unwrap(),
            "string".to_string()
        );

        assert!(mapping_to_check
            .get_mapping("dict", false)
            .expect("")
            .get_array("array", false)
            .unwrap()
            .contains_item(&toml_edit::Item::from(toml_edit::Value::from(1))));

        mapping_to_check
            .get_mapping("new_dict", true)
            .unwrap()
            .insert(
                "key",
                &toml_edit::Item::Value(toml_edit::Value::from("new_dict_value")),
            );

        assert_eq!(
            mapping_to_check
                .get_mapping("new_dict", false)
                .expect("")
                .get_string("key")
                .unwrap(),
            "new_dict_value".to_string()
        );

        mapping_to_check
            .get_mapping("dict", false)
            .unwrap()
            .get_mapping("new_nested_dict", true)
            .unwrap()
            .insert(
                "key",
                &toml_edit::Item::Value(toml_edit::Value::from("new_nested_dict_value")),
            );

        assert_eq!(
            mapping_to_check
                .get_mapping("dict", false)
                .unwrap()
                .get_mapping("new_nested_dict", false)
                .unwrap()
                .get_string("key")
                .unwrap(),
            "new_nested_dict_value".to_string()
        );

        mapping_to_check
            .get_array("new_array", true)
            .unwrap()
            .insert_when_not_present(&toml_edit::Item::from(toml_edit::Value::from(
                "new_array_value",
            )));

        assert!(mapping_to_check
            .get_array("new_array", false)
            .unwrap()
            .contains_item(&toml_edit::Item::from(toml_edit::Value::from(
                "new_array_value"
            ))));

        mapping_to_check
            .get_mapping("dict", false)
            .unwrap()
            .get_array("new_nested_array", true)
            .unwrap()
            .insert_when_not_present(&toml_edit::Item::from(toml_edit::Value::from(
                "new_nested_array_value",
            )));

        assert!(mapping_to_check
            .get_mapping("dict", false)
            .unwrap()
            .get_array("new_nested_array", false)
            .unwrap()
            .contains_item(&toml_edit::Item::from(toml_edit::Value::from(
                "new_nested_array_value"
            ))));
    }
}
