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
    fn insert(&mut self, key: &str, value: &toml::Value);
    fn remove(&mut self, key: &str);
}

pub(crate) trait Array {
    fn insert_when_not_present(&mut self, value: &toml::Value);

    fn remove(&mut self, value: &toml::Value);

    fn contains_item(&self, value: &toml::Value) -> bool;
}
pub(crate) trait Value {
    fn from_toml_value(value: &toml::Value) -> Self
    where
        Self: Sized;
}

#[cfg(test)]
pub(crate) mod tests {

    use super::Mapping;

    pub(crate) fn get_test_table() -> toml::Value {
        let mut table = toml::Table::new();
        table.insert("str".to_owned(), toml::Value::String("string".to_string()));
        table.insert("int".to_owned(), toml::Value::Integer(1));
        table.insert("float".to_owned(), toml::Value::Float(1.1));
        table.insert("bool".to_owned(), toml::Value::Boolean(true));
        table.insert(
            "array".to_owned(),
            toml::Value::Array(vec![toml::Value::Integer(1)]),
        );
        let nested_table = table.clone();
        table.insert("dict".to_owned(), nested_table.into());

        toml::Value::Table(table)
    }

    pub(crate) fn test_mapping(mut mapping_to_check: Box<dyn Mapping>) {
        assert!(mapping_to_check
            .get_array("array", false)
            .expect("")
            .contains_item(&toml::Value::Integer(1)));

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
            .contains_item(&toml::Value::Integer(1)),);

        mapping_to_check
            .get_mapping("new_dict", true)
            .unwrap()
            .insert("key", &toml::Value::String("new_dict_value".to_string()));

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
                &toml::Value::String("new_nested_dict_value".to_string()),
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
            .insert_when_not_present(&toml::Value::String("new_array_value".to_string()));

        assert!(mapping_to_check
            .get_array("new_array", false)
            .unwrap()
            .contains_item(&toml::Value::String("new_array_value".to_string())),);

        mapping_to_check
            .get_mapping("dict", false)
            .unwrap()
            .get_array("new_nested_array", true)
            .unwrap()
            .insert_when_not_present(&toml::Value::String("new_nested_array_value".to_string()));

        assert!(mapping_to_check
            .get_mapping("dict", false)
            .unwrap()
            .get_array("new_nested_array", false)
            .unwrap()
            .contains_item(&toml::Value::String("new_nested_array_value".to_string())),);
    }
}
