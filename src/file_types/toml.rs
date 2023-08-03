use std::fmt::format;

use super::FileType;
use regex::Regex;
use toml_edit::Document;
pub(crate) struct Toml {}

impl Toml {
    pub fn new() -> Toml {
        Toml {}
    }
}

impl FileType for Toml {
    fn set(&self, contents: &str, table_to_set: &toml::Table) -> Result<String, String> {
        set(contents, table_to_set)
    }

    fn unset(&self, contents: &str, table_to_unset: &toml::Table) -> Result<String, String> {
        unset(contents, table_to_unset)
    }

    fn validate_regex(&self, contents: &str, table_with_regex: &toml::Table) -> Result<(), String> {
        validate_regex(contents, table_with_regex)
    }
}

fn validate_regex(contents: &str, table_with_regex: &toml::Table) -> Result<(), String> {
    let mut doc = contents.parse::<Document>().unwrap();
    let doc_table = doc.as_table_mut();
    _validate_key_regex(doc_table, table_with_regex)
}

fn _validate_key_regex(
    doc: &mut toml_edit::Table,
    table_with_regex: &toml::Table,
) -> Result<(), String> {
    for (k, v) in table_with_regex {
        match v {
            toml::Value::String(raw_regex) => match doc.get(k) {
                Some(toml_edit::Item::Value(toml_edit::Value::String(string_to_match))) => {
                    let regex = match Regex::new(raw_regex) {
                        Ok(regex) => regex,
                        Err(s) => return Err(s.to_string()),
                    };
                    if regex.is_match(string_to_match.value()) {
                        return Ok(());
                    } else {
                        return Err(format!(
                            "Regex does not match. key: {}, regex: {}, value: {}",
                            k,
                            raw_regex,
                            string_to_match.value()
                        ));
                    }
                }
                _ => return Err("key does not exists".to_string()),
            },
            toml::Value::Table(t) => match doc.get_mut(k) {
                Some(toml_edit::Item::Table(child_doc)) => {
                    return _validate_key_regex(child_doc, t)
                }
                _ => return Err("key does not exists".to_string()),
            },

            _ => {}
        }
    }
    Err("".to_string())
}

fn set(contents: &str, table_to_set: &toml::Table) -> Result<String, String> {
    let mut doc = contents.parse::<Document>().unwrap();
    let doc_table = doc.as_table_mut();

    _set_key_value(doc_table, table_to_set);

    Ok(doc.to_string())
}

fn _convert_value_to_item(value: &toml::Value) -> toml_edit::Value {
    match value {
        toml::Value::String(v) => toml_edit::Value::String(toml_edit::Formatted::new(v.clone())),
        toml::Value::Integer(v) => toml_edit::Value::Integer(toml_edit::Formatted::new(*v)),
        toml::Value::Float(v) => toml_edit::Value::Float(toml_edit::Formatted::new(*v)),
        toml::Value::Boolean(v) => toml_edit::Value::Boolean(toml_edit::Formatted::new(*v)),
        toml::Value::Datetime(v) => toml_edit::Value::Datetime(toml_edit::Formatted::new(*v)),
        toml::Value::Array(v) => {
            let mut a = toml_edit::Array::new();
            for v_item in v {
                a.push_formatted(_convert_value_to_item(v_item))
            }
            toml_edit::Value::Array(a)
        }
        toml::Value::Table(v) => {
            let mut a = toml_edit::InlineTable::new();
            for (k, v_item) in v {
                a.insert(k, _convert_value_to_item(v_item));
            }

            toml_edit::Value::InlineTable(a)
        }
    }
}

fn _set_key_value(doc: &mut toml_edit::Table, table_to_set: &toml::Table) {
    for (k, v) in table_to_set {
        if !v.is_table() {
            doc[k] = toml_edit::Item::Value(_convert_value_to_item(v));
            continue;
        }
        if !doc.contains_key(k) {
            doc[k] = toml_edit::Item::Table(toml_edit::Table::new());
        }
        let child_doc = doc.get_mut(k).unwrap();
        if !child_doc.is_table() {
            panic!("Unexpected value type");
        }
        _set_key_value(child_doc.as_table_mut().unwrap(), v.as_table().unwrap());
    }
}

fn unset(contents: &str, table_to_unset: &toml::Table) -> Result<String, String> {
    // remove all the keys in the table where the key is the end node
    let mut doc = contents.parse::<Document>().unwrap();
    let doc_table = doc.as_table_mut();

    _remove_key(doc_table, table_to_unset);

    Ok(doc.to_string())
}

fn _remove_key(doc: &mut toml_edit::Table, table_to_unset: &toml::Table) {
    for (key_to_unset, value_to_unset) in table_to_unset {
        if let toml::Value::Table(child_table_to_unset) = value_to_unset {
            if child_table_to_unset.is_empty() {
                doc.remove(key_to_unset);
            } else if let Some(child_doc) = doc.get_mut(key_to_unset) {
                if let Some(child_doc_table) = child_doc.as_table_mut() {
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

    #[test]
    fn test_unset() {
        let contents = r#"[package]
name = "foo"
version = "1.0"

[dependencies]
bar = { version = "1.0" }
foo = "1.0"
"#;

        let contents_to_unset = r#"
[dependencies.bar]
"#;

        let contents_to_unset = toml::from_str::<toml::Value>(contents_to_unset).unwrap();
        let contents_to_unset = contents_to_unset.as_table().unwrap();

        let new_contents = super::unset(contents, contents_to_unset).unwrap();

        assert_eq!(
            new_contents,
            r#"[package]
name = "foo"
version = "1.0"

[dependencies]
foo = "1.0"
"#
        );
    }

    #[test]
    fn test_set_simple() {
        let contents = r#"[package]
name = "foo"
version = "1.0"
"#;

        let contents_to_set = r#"[package]
name = "bar"
"#;

        let contents_to_set = toml::from_str::<toml::Value>(contents_to_set).unwrap();
        let contents_to_set = contents_to_set.as_table().unwrap();

        let new_contents = super::set(contents, contents_to_set).unwrap();

        assert_eq!(
            new_contents,
            r#"[package]
name = "bar"
version = "1.0"
"#
        );
    }

    #[test]
    fn test_set_nested() {
        let contents = r#"[package]
name = "foo"
version = "1.0"

[dependencies]
toml ="1.0"

[dependencies.bar]
version = "1.0"
features = ["foo"]
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
            r#"[package]
name = "foo"
version = "1.0"

[dependencies]
toml ="1.0"

[dependencies.bar]
version = "2.0"
features = ["bar"]
"#
        );
    }

    #[test]
    fn test_regex() {
        let contents = r#"[package]
name = "foo"
version = "1.0"

[dependencies]
toml ="1.0"

[dependencies.bar]
version = "1.0"
features = ["foo"]
"#;

        let contents_with_matched_regex = r#"[dependencies.bar]
version = "[0-9]\\.[0-9]"
"#;

        let contents_with_matched_regex =
            toml::from_str::<toml::Value>(contents_with_matched_regex).unwrap();
        let contents_with_matched_regex = contents_with_matched_regex.as_table().unwrap();

        assert_eq!(
            super::validate_regex(contents, contents_with_matched_regex),
            Ok(())
        );

        let contents_with_unmatched_regex = r#"[dependencies.bar]
version = "[0-9][0-9]"
"#;

        let contents_with_unmatched_regex =
            toml::from_str::<toml::Value>(contents_with_unmatched_regex).unwrap();
        let contents_with_unmatched_regex = contents_with_unmatched_regex.as_table().unwrap();

        assert_eq!(
            super::validate_regex(contents, contents_with_unmatched_regex),
            Err("Regex does not match".to_string())
        );
    }
}
