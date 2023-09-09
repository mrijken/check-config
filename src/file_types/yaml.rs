use crate::checkers::base::CheckError;

use super::{generic, FileType, RegexValidateResult};

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
    fn set(&self, contents: &str, table_to_set: &toml::Table) -> Result<String, CheckError> {
        let mut doc = convert_string(contents)?;

        // todo convert unwrap
        generic::set_key_value(&mut doc, table_to_set);

        Ok(serde_yaml::to_string(&doc).unwrap())
    }

    fn unset(&self, contents: &str, table_to_unset: &toml::Table) -> Result<String, CheckError> {
        let mut doc = convert_string(contents)?;

        // todo convert unwrap
        generic::unset_key(&mut doc, table_to_unset);

        Ok(serde_yaml::to_string(&doc).unwrap())
    }

    fn validate_key_value_regex(
        &self,
        contents: &str,
        table_with_regex: &toml::Table,
    ) -> Result<RegexValidateResult, CheckError> {
        let mut doc = convert_string(contents)?;
        generic::validate_key_value_regex(&mut doc, table_with_regex, "".to_string())
    }
}

fn convert_string(contents: &str) -> Result<Mapping, CheckError> {
    if contents.trim().is_empty() {
        return Ok(Mapping::new());
    }
    let doc: Value =
        serde_yaml::from_str(contents).map_err(|e| CheckError::InvalidFileFormat(e.to_string()))?;
    Ok(doc
        .as_mapping()
        .ok_or(CheckError::InvalidFileFormat("No object".to_string()))?
        .clone())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_types::RegexValidateResult;

    #[test]
    fn test_unset() {
        let contents = r#"
package:
  name: "foo"
  version: "1.0"
dependencies:
  bar:
    version: "1.0"
  foo: "1.0"
"#;

        let contents_to_unset = r#"
[dependencies.bar]
"#;

        let contents_to_unset = toml::from_str::<toml::Value>(contents_to_unset).unwrap();
        let contents_to_unset = contents_to_unset.as_table().unwrap();

        let new_contents = super::Yaml::new()
            .unset(contents, contents_to_unset)
            .unwrap();

        assert_eq!(
            new_contents,
            r#"package:
  name: foo
  version: '1.0'
dependencies:
  foo: '1.0'
"#
        );
    }

    #[test]

    fn test_set_simple() {
        let contents = r#"
package:
  name: "foo"
  version: "1.0"
"#;

        let contents_to_set = r#"[package]
name = "bar"
"#;

        let contents_to_set = toml::from_str::<toml::Value>(contents_to_set).unwrap();
        let contents_to_set = contents_to_set.as_table().unwrap();

        let new_contents = super::Yaml::new().set(contents, contents_to_set).unwrap();

        assert_eq!(
            new_contents,
            r#"package:
  name: bar
  version: '1.0'
"#
        );
    }

    #[test]
    fn test_set_nested() {
        let contents = r#"
package:
  name: "foo"
  version: "1.0"
dependencies:
  bar:
    version: "1.0"
    features:
    - foo
  toml: "1.0"
"#;

        let contents_to_set = r#"[dependencies.bar]
version = "2.0"
features = ["bar"]
"#;

        let contents_to_set = toml::from_str::<toml::Value>(contents_to_set).unwrap();
        let contents_to_set = contents_to_set.as_table().unwrap();

        let new_contents = super::Yaml::new().set(contents, contents_to_set).unwrap();

        assert_eq!(
            new_contents,
            r#"package:
  name: foo
  version: '1.0'
dependencies:
  bar:
    version: '2.0'
    features:
    - bar
  toml: '1.0'
"#
        );
    }

    #[test]
    fn test_regex() {
        let contents = r#"
{
    "package": {
        "name": "foo",
        "version": "1.0"
    },
    "dependencies": {
        "bar": {
            "version": "1.0",
            "features": ["foo"]
        },
        "toml": "1.0"
    }
}
"#;

        let contents_with_matched_regex = r#"[dependencies.bar]
version = "[0-9]\\.[0-9]"
"#;

        let contents_with_matched_regex =
            toml::from_str::<toml::Value>(contents_with_matched_regex).unwrap();
        let contents_with_matched_regex = contents_with_matched_regex.as_table().unwrap();

        assert_eq!(
            super::Yaml::new()
                .validate_key_value_regex(contents, contents_with_matched_regex)
                .unwrap(),
            RegexValidateResult::Valid
        );

        let contents_with_unmatched_regex = r#"[dependencies.bar]
version = "[0-9][0-9]"
"#;

        let contents_with_unmatched_regex =
            toml::from_str::<toml::Value>(contents_with_unmatched_regex).unwrap();
        let contents_with_unmatched_regex = contents_with_unmatched_regex.as_table().unwrap();

        assert_eq!(
            super::Yaml::new()
                .validate_key_value_regex(contents, contents_with_unmatched_regex)
                .unwrap(),
            RegexValidateResult::Invalid {
                key: "dependencies.bar.version".to_string(),
                regex: "[0-9][0-9]".to_string(),
                found: "1.0".to_string()
            }
        );
    }
}
