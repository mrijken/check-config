use regex::Regex;

use crate::{file_types::RegexValidateResult, mapping::generic::Mapping};

use super::{
    base::{Action, Check, CheckConstructor, CheckError},
    GenericCheck,
};

#[derive(Debug)]
pub(crate) struct EntryRegexMatch {
    generic_check: GenericCheck,
    value: toml_edit::Table,
}

impl CheckConstructor for EntryRegexMatch {
    type Output = Self;

    fn from_check_table(
        generic_check: GenericCheck,
        value: toml_edit::Table,
    ) -> Result<Self::Output, super::base::CheckDefinitionError> {
        Ok(Self {
            generic_check,
            value,
        })
    }
}
impl Check for EntryRegexMatch {
    fn check_type(&self) -> String {
        "key_value_regex_match".to_string()
    }

    fn generic_check(&self) -> &GenericCheck {
        &self.generic_check
    }

    fn get_action(&self) -> Result<Action, CheckError> {
        let mut doc = self.generic_check().get_mapping()?;

        match validate_key_value_regex(doc.as_mut(), &self.value, "".to_string()) {
            Ok(RegexValidateResult::Valid) => Ok(Action::None),
            Ok(RegexValidateResult::Invalid {
                key,
                regex,
                found: _,
            }) => Ok(Action::MatchKeyRegex { key, regex }),
            Err(e) => Err(CheckError::InvalidRegex(e.to_string())),
        }
    }
}

fn make_key_path(parent: &str, key: &str) -> String {
    if parent.is_empty() {
        key.to_string()
    } else {
        parent.to_string() + "." + key
    }
}

fn validate_key_value_regex(
    doc: &mut dyn Mapping,
    table_with_regex: &toml_edit::Table,
    key_path: String,
) -> Result<RegexValidateResult, CheckError> {
    for (key, value) in table_with_regex {
        match value {
            toml_edit::Item::Value(toml_edit::Value::String(raw_regex)) => {
                match doc.get_string(key) {
                    Ok(string_to_match) => {
                        let regex = match Regex::new(raw_regex.value()) {
                            Ok(regex) => regex,
                            Err(s) => return Err(CheckError::InvalidRegex(s.to_string())),
                        };
                        if regex.is_match(string_to_match.as_str()) {
                            return Ok(RegexValidateResult::Valid);
                        } else {
                            return Ok(RegexValidateResult::Invalid {
                                key: make_key_path(&key_path, key),
                                regex: raw_regex.value().to_owned(),
                                found: string_to_match.clone(),
                            });
                        }
                    }
                    _ => {
                        return Ok(RegexValidateResult::Invalid {
                            key: make_key_path(&key_path, key),
                            regex: raw_regex.value().to_owned(),
                            found: "".to_string(),
                        })
                    }
                }
            }
            toml_edit::Item::Table(t) => match doc.get_mapping(key, false) {
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
mod tests {
    use crate::checkers::test_helpers::read_test_files;

    use super::*;

    #[test]
    fn test_test_files() {
        for (test_path, test_input, test_expected_output, checker) in
            read_test_files("key_value_regex_match")
        {
            let mut test_input = test_input;
            let result =
                validate_key_value_regex(test_input.as_mut(), &checker, "".to_string()).unwrap();

            if test_expected_output.contains("true") {
                assert_eq!(
                    result,
                    RegexValidateResult::Valid,
                    "test_path {test_path} failed"
                );
            } else {
                assert_ne!(
                    result,
                    RegexValidateResult::Valid,
                    "test_path {test_path} failed"
                );
            }
        }
    }
}
