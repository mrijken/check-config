use crate::checkers::base::CheckError;

#[derive(Debug)]
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
