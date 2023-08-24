use crate::checkers::base::CheckError;

use super::{FileType, RegexValidateResult};
use regex::Regex;
use serde_json::{Map, Value};

pub(crate) struct Json {}

impl Json {
    pub fn new() -> Json {
        Json {}
    }
}

impl FileType for Json {
    fn set(&self, contents: &str, table_to_set: &toml::Table) -> Result<String, CheckError> {
        set(contents, table_to_set)
    }

    fn unset(&self, contents: &str, table_to_unset: &toml::Table) -> Result<String, CheckError> {
        unset(contents, table_to_unset)
    }

    fn remove_entries(
        &self,
        contents: &str,
        entries_to_remove: &toml::Table,
    ) -> Result<String, CheckError> {
        remove_entries(contents, entries_to_remove)
    }

    fn add_entries(
        &self,
        contents: &str,
        entries_to_add: &toml::Table,
    ) -> Result<String, CheckError> {
        add_entries(contents, entries_to_add)
    }

    fn validate_regex(
        &self,
        contents: &str,
        table_with_regex: &toml::Table,
    ) -> Result<RegexValidateResult, CheckError> {
        validate_regex(contents, table_with_regex)
    }
}

fn add_entries(
    contents: &str,
    entries_to_add: &toml::map::Map<String, toml::Value>,
) -> Result<String, CheckError> {
    let mut doc = convert_string(contents)?;
    _add_entries(&mut doc, entries_to_add);
    Ok(serde_json::to_string_pretty(&doc).unwrap())
}

fn _add_entries(
    doc: &mut Map<String, Value>,
    entries_to_add: &toml::map::Map<String, toml::Value>,
) {
    for (k, v) in entries_to_add {
        if !v.is_table() {
            panic!("Unexpected value type");
        }
        let v = v.as_table().unwrap();
        if v.contains_key("__items__") {
            if doc.contains_key(k) {
                if !doc.get(k).unwrap().is_array() {
                    panic!("Expecting array");
                }
            } else {
                doc.insert(k.clone(), Value::Array(vec![]));
            }

            let doc_array = doc.get_mut(k).unwrap().as_array_mut().unwrap();

            for item in v.get("__items__").unwrap().as_array().unwrap() {
                let item = _convert_value_to_item(item);
                if !doc_array.contains(&item) {
                    doc_array.push(item);
                }
            }
            continue;
        }

        if !doc.contains_key(k) {
            doc.insert(k.clone(), Value::Object(Map::new()));
        }
        let child_doc = doc.get_mut(k).unwrap();
        if !child_doc.is_object() {
            panic!("Unexpected value type");
        }
        _add_entries(child_doc.as_object_mut().unwrap(), v);
    }
}

fn remove_entries(
    contents: &str,
    entries_to_remove: &toml::map::Map<String, toml::Value>,
) -> Result<String, CheckError> {
    let mut doc = convert_string(contents)?;
    _remove_entries(&mut doc, entries_to_remove);
    Ok(serde_json::to_string_pretty(&doc).unwrap())
}

fn _remove_entries(
    doc: &mut Map<String, Value>,
    entries_to_remove: &toml::map::Map<String, toml::Value>,
) {
    for (k, v) in entries_to_remove {
        if !v.is_table() {
            panic!("Unexpected value type");
        }
        let v = v.as_table().unwrap();
        if v.contains_key("__items__") {
            if !doc.contains_key(k) || !doc.get(k).unwrap().is_array() {
                return;
            }

            let doc_array = doc.get_mut(k).unwrap().as_array_mut().unwrap();

            for item in v.get("__items__").unwrap().as_array().unwrap() {
                let item = _convert_value_to_item(item);
                if let Some(idx) = json_edit_array_index(doc_array, &item) {
                    doc_array.remove(idx);
                }
            }
            continue;
        }

        let child_doc = doc.get_mut(k).unwrap();
        if !child_doc.is_object() {
            panic!("Unexpected value type");
        }
        _remove_entries(child_doc.as_object_mut().unwrap(), v);
    }
}

fn json_edit_array_index(array: &[Value], item: &Value) -> Option<usize> {
    for (idx, array_item) in array.iter().enumerate() {
        if array_item == item {
            return Some(idx);
        }
    }
    None
}

fn convert_string(contents: &str) -> Result<Map<String, Value>, CheckError> {
    if contents.trim().is_empty() {
        return Ok(Map::new());
    }
    let doc: Value =
        serde_json::from_str(contents).map_err(|e| CheckError::InvalidFileFormat(e.to_string()))?;
    let doc = doc
        .as_object()
        .ok_or(CheckError::InvalidFileFormat("No object".to_string()))?;
    Ok(doc.clone())
}
pub(crate) fn make_key_path(parent: &str, key: &str) -> String {
    if parent.is_empty() {
        key.to_string()
    } else {
        parent.to_string() + "." + key
    }
}
fn validate_regex(
    contents: &str,
    table_with_regex: &toml::Table,
) -> Result<RegexValidateResult, CheckError> {
    let mut doc = convert_string(contents)?;
    _validate_key_regex(&mut doc, table_with_regex, "".to_string())
}

fn _validate_key_regex(
    doc: &mut Map<String, Value>,
    table_with_regex: &toml::Table,
    key_path: String,
) -> Result<RegexValidateResult, CheckError> {
    for (k, v) in table_with_regex {
        match v {
            toml::Value::String(raw_regex) => match doc.get(k) {
                Some(Value::String(string_to_match)) => {
                    let regex = match Regex::new(raw_regex) {
                        Ok(regex) => regex,
                        Err(s) => return Err(CheckError::InvalidRegex(s.to_string())),
                    };
                    if regex.is_match(string_to_match) {
                        return Ok(RegexValidateResult::Valid);
                    } else {
                        return Ok(RegexValidateResult::Invalid {
                            key: make_key_path(&key_path, k),
                            regex: raw_regex.clone(),
                            found: string_to_match.clone(),
                        });
                    }
                }
                _ => {
                    return Ok(RegexValidateResult::Invalid {
                        key: make_key_path(&key_path, k),
                        regex: raw_regex.clone(),
                        found: "".to_string(),
                    })
                }
            },
            toml::Value::Table(t) => match doc.get_mut(k) {
                Some(Value::Object(child_doc)) => {
                    return _validate_key_regex(child_doc, t, make_key_path(&key_path, k))
                }
                _ => {
                    return Ok(RegexValidateResult::Invalid {
                        key: make_key_path(&key_path, k),
                        regex: "".to_string(),
                        found: "".to_string(),
                    })
                }
            },

            _ => {}
        }
    }
    Ok(RegexValidateResult::Valid)
}

fn set(contents: &str, table_to_set: &toml::Table) -> Result<String, CheckError> {
    let mut doc = convert_string(contents)?;

    _set_key_value(&mut doc, table_to_set);

    Ok(serde_json::to_string_pretty(&doc).unwrap())
}

fn _convert_value_to_item(value: &toml::Value) -> Value {
    match value {
        toml::Value::String(v) => Value::String(v.clone()),
        toml::Value::Integer(v) => Value::Number(serde_json::Number::from(*v)),
        toml::Value::Float(v) => Value::Number(serde_json::Number::from_f64(*v).unwrap()),
        toml::Value::Boolean(v) => Value::Bool(*v),
        toml::Value::Datetime(v) => Value::String(v.to_string()),
        toml::Value::Array(v) => {
            let mut a = vec![];
            for v_item in v {
                a.push(_convert_value_to_item(v_item))
            }
            Value::Array(a)
        }
        toml::Value::Table(v) => {
            let mut a: Map<String, Value> = Map::new();
            for (k, v_item) in v {
                a.insert(k.clone(), _convert_value_to_item(v_item));
            }

            Value::Object(a)
        }
    }
}

fn _set_key_value(doc: &mut Map<String, Value>, table_to_set: &toml::Table) {
    for (k, v) in table_to_set {
        if !v.is_table() {
            doc.insert(k.clone(), _convert_value_to_item(v));
            continue;
        }
        if !doc.contains_key(k) {
            doc.insert(k.clone(), Value::Object(Map::new()));
        }
        let child_doc = doc.get_mut(k).unwrap();
        if !child_doc.is_object() {
            panic!("Unexpected value type");
        }
        _set_key_value(child_doc.as_object_mut().unwrap(), v.as_table().unwrap());
    }
}

fn unset(contents: &str, table_to_unset: &toml::Table) -> Result<String, CheckError> {
    // remove all the keys in the table where the key is the end node
    let mut doc = convert_string(contents)?;

    _remove_key(&mut doc, table_to_unset);

    Ok(serde_json::to_string_pretty(&doc).unwrap())
}

fn _remove_key(doc: &mut Map<String, Value>, table_to_unset: &toml::Table) {
    for (key_to_unset, value_to_unset) in table_to_unset {
        if let toml::Value::Table(child_table_to_unset) = value_to_unset {
            if child_table_to_unset.is_empty() {
                doc.remove(key_to_unset);
            } else if let Some(child_doc) = doc.get_mut(key_to_unset) {
                if let Some(child_doc_table) = child_doc.as_object_mut() {
                    _remove_key(child_doc_table, child_table_to_unset);
                };
            } else {
                log::info!(
                    "Key {} is not found in toml, so we can not remove that key",
                    key_to_unset,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::file_types::RegexValidateResult;

    #[test]
    fn test_add_entries() {
        let contents = r#"{"key": {
        "list": [1,2]}
    }
"#;

        let entries_to_add = r#"
[key.list]
__items__ = [2, 3, 4]

[key2.list]
__items__ = [2, 3, 4]
"#;

        let entries_to_add = toml::from_str::<toml::Value>(entries_to_add).unwrap();
        let entries_to_add = entries_to_add.as_table().unwrap();

        let new_contents = super::add_entries(contents, entries_to_add).unwrap();

        assert_eq!(
            new_contents,
            "{\n  \"key\": {\n    \"list\": [\n      1,\n      2,\n      3,\n      4\n    ]\n  },\n  \"key2\": {\n    \"list\": [\n      2,\n      3,\n      4\n    ]\n  }\n}"
                .to_string()
        );
    }

    #[test]
    fn test_add_entries_with_tables() {
        let contents = r#"{"key": {
  "list": [
  { "key": "1"} ,
  { "key": "2"}]
  }
  }
"#;

        let entries_to_add = r#"
[key.list]
__items__ = [{key = "3"}, {key = "2"}, {key = "4"}]
"#;

        let entries_to_add = toml::from_str::<toml::Value>(entries_to_add).unwrap();
        let entries_to_add = entries_to_add.as_table().unwrap();

        let new_contents = super::add_entries(contents, entries_to_add).unwrap();

        assert_eq!(
            new_contents,
            "{\n  \"key\": {\n    \"list\": [\n      {\n        \"key\": \"1\"\n      },\n      {\n        \"key\": \"2\"\n      },\n      {\n        \"key\": \"3\"\n      },\n      {\n        \"key\": \"4\"\n      }\n    ]\n  }\n}".to_string()
        );
    }

    #[test]
    fn test_remove_entries() {
        let contents = r#"{"key": {
  "list": [1,2]
  }}
"#;

        let entries_to_remove = r#"
[key.list]
__items__ = [2, 3, 4]
"#;

        let entries_to_remove = toml::from_str::<toml::Value>(entries_to_remove).unwrap();
        let entries_to_remove = entries_to_remove.as_table().unwrap();

        let new_contents = super::remove_entries(contents, entries_to_remove).unwrap();

        assert_eq!(
            new_contents,
            "{\n  \"key\": {\n    \"list\": [\n      1\n    ]\n  }\n}".to_string()
        );
    }

    #[test]
    fn test_unset() {
        let contents = r#"
{
    "package": {
        "name": "foo",
        "version": "1.0"
    },
    "dependencies": {
        "bar": {
            "version": "1.0"
        },
        "foo": "1.0"
    }
}
"#;

        let contents_to_unset = r#"
[dependencies.bar]
"#;

        let contents_to_unset = toml::from_str::<toml::Value>(contents_to_unset).unwrap();
        let contents_to_unset = contents_to_unset.as_table().unwrap();

        let new_contents = super::unset(contents, contents_to_unset).unwrap();

        assert_eq!(
            new_contents,
            r#"{
  "dependencies": {
    "foo": "1.0"
  },
  "package": {
    "name": "foo",
    "version": "1.0"
  }
}"#
        );
    }

    #[test]

    fn test_set_simple() {
        let contents = r#"{
    "package": {
        "name": "foo",
        "version": "1.0"
    }
}
"#;

        let contents_to_set = r#"[package]
name = "bar"
"#;

        let contents_to_set = toml::from_str::<toml::Value>(contents_to_set).unwrap();
        let contents_to_set = contents_to_set.as_table().unwrap();

        let new_contents = super::set(contents, contents_to_set).unwrap();

        assert_eq!(
            new_contents,
            r#"{
  "package": {
    "name": "bar",
    "version": "1.0"
  }
}"#
        );
    }

    #[test]
    fn test_set_nested() {
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

        let contents_to_set = r#"[dependencies.bar]
version = "2.0"
features = ["bar"]
"#;

        let contents_to_set = toml::from_str::<toml::Value>(contents_to_set).unwrap();
        let contents_to_set = contents_to_set.as_table().unwrap();

        let new_contents = super::set(contents, contents_to_set).unwrap();

        assert_eq!(
            new_contents,
            r#"{
  "dependencies": {
    "bar": {
      "features": [
        "bar"
      ],
      "version": "2.0"
    },
    "toml": "1.0"
  },
  "package": {
    "name": "foo",
    "version": "1.0"
  }
}"#
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
            super::validate_regex(contents, contents_with_matched_regex).unwrap(),
            RegexValidateResult::Valid
        );

        let contents_with_unmatched_regex = r#"[dependencies.bar]
version = "[0-9][0-9]"
"#;

        let contents_with_unmatched_regex =
            toml::from_str::<toml::Value>(contents_with_unmatched_regex).unwrap();
        let contents_with_unmatched_regex = contents_with_unmatched_regex.as_table().unwrap();

        assert_eq!(
            super::validate_regex(contents, contents_with_unmatched_regex).unwrap(),
            RegexValidateResult::Invalid {
                key: "dependencies.bar.version".to_string(),
                regex: "[0-9][0-9]".to_string(),
                found: "1.0".to_string()
            }
        );
    }
}
