use crate::checkers::base::CheckError;

use super::FileType;

use serde_yaml::{Mapping, Value};

pub(crate) struct Yaml {}

impl Yaml {
    pub fn new() -> Yaml {
        Yaml {}
    }
}

impl FileType for Yaml {
    fn to_mapping(
        &self,
        contents: &str,
    ) -> Result<Box<dyn crate::mapping::generic::Mapping>, CheckError> {
        crate::mapping::yaml::from_string(contents)
    }
}

fn _convert_value_to_item(value: &toml::Value) -> Value {
    match value {
        toml::Value::String(v) => Value::String(v.clone()),
        toml::Value::Integer(v) => Value::Number(serde_yaml::Number::from(*v)),
        toml::Value::Float(v) => Value::Number(serde_yaml::Number::from(*v)),
        toml::Value::Boolean(v) => Value::Bool(*v),
        toml::Value::Datetime(v) => Value::String(v.to_string()),
        toml::Value::Array(v) => {
            let mut a = vec![];
            for v_item in v {
                a.push(_convert_value_to_item(v_item))
            }
            Value::Sequence(a)
        }
        toml::Value::Table(v) => {
            let mut a: Mapping = Mapping::new();
            for (k, v_item) in v {
                a.insert(Value::String(k.clone()), _convert_value_to_item(v_item));
            }

            Value::Mapping(a)
        }
    }
}
