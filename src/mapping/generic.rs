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
}
