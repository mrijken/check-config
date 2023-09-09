use regex::Regex;

use crate::{checkers::base::CheckError, mapping::generic::Mapping};

use super::RegexValidateResult;

fn make_key_path(parent: &str, key: &str) -> String {
    if parent.is_empty() {
        key.to_string()
    } else {
        parent.to_string() + "." + key
    }
}

pub(crate) fn set_key_value(doc: &mut dyn Mapping, table_to_set: &toml::Table) {
    for (k, v) in table_to_set {
        if !v.is_table() {
            dbg!(k, v);
            doc.insert(k, v);
            continue;
        }
        let child_doc = doc.get_mapping(k, true).unwrap();
        set_key_value(child_doc, v.as_table().unwrap());
    }
}

pub(crate) fn unset_key(doc: &mut dyn Mapping, table_to_unset: &toml::Table) {
    for (key_to_unset, value_to_unset) in table_to_unset {
        if let toml::Value::Table(child_table_to_unset) = value_to_unset {
            if child_table_to_unset.is_empty() {
                doc.remove(key_to_unset);
            } else if let Ok(child_doc) = doc.get_mapping(key_to_unset, false) {
                unset_key(child_doc, child_table_to_unset);
            } else {
                log::info!(
                    "Key {} is not found in toml, so we can not remove that key",
                    key_to_unset,
                );
            }
        }
    }
}

pub(crate) fn validate_key_value_regex(
    doc: &mut dyn Mapping,
    table_with_regex: &toml::Table,
    key_path: String,
) -> Result<RegexValidateResult, CheckError> {
    for (key, value) in table_with_regex {
        match value {
            toml::Value::String(raw_regex) => match doc.get_string(key) {
                Ok(string_to_match) => {
                    let regex = match Regex::new(raw_regex) {
                        Ok(regex) => regex,
                        Err(s) => return Err(CheckError::InvalidRegex(s.to_string())),
                    };
                    if regex.is_match(string_to_match.as_str()) {
                        return Ok(RegexValidateResult::Valid);
                    } else {
                        return Ok(RegexValidateResult::Invalid {
                            key: make_key_path(&key_path, key),
                            regex: raw_regex.clone(),
                            found: string_to_match.clone(),
                        });
                    }
                }
                _ => {
                    return Ok(RegexValidateResult::Invalid {
                        key: make_key_path(&key_path, key),
                        regex: raw_regex.clone(),
                        found: "".to_string(),
                    })
                }
            },
            toml::Value::Table(t) => match doc.get_mapping(key, false) {
                Ok(child_doc) => {
                    return validate_key_value_regex(child_doc, t, make_key_path(&key_path, key))
                }
                _ => {
                    return Ok(RegexValidateResult::Invalid {
                        key: make_key_path(&key_path, key),
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

#[cfg(test)]
mod test {

    use super::*;
    use crate::mapping::generic::MappingError;

    #[test]
    fn test_mapping() {
        let contents = r#"{
            "1": {
                "1.1": 1
            },
            "2": {
                "2.1": "2"
            }

        }
        "#;
        let mut doc: serde_json::Value = serde_json::from_str(contents).unwrap();
        let map = doc.as_object_mut().unwrap();

        let key3 = "3";

        // check existing sub map
        assert!(map.contains_key("1"));
        assert!(map
            .get_mapping("1", false)
            .ok()
            .unwrap()
            .contains_key("1.1"),);
        assert!(map
            .get_mapping("2", false)
            .ok()
            .unwrap()
            .contains_key("2.1"),);
        matches!(
            map.get_mapping("2", false)
                .ok()
                .unwrap()
                .get_string("2.1-absent"),
            Err(MappingError::MissingKey(key3))
        );

        // Add new sub map

        assert!(!map.contains_key(key3));
        matches!(
            map.get_array(key3, false).err().unwrap(),
            MappingError::MissingKey(key3)
        );
        assert!(!map.contains_key(key3));
        assert!(map.get_array(key3, true).is_ok());

        assert!(map.contains_key(key3));
    }
}
